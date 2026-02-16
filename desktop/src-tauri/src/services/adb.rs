use crate::models::device::RawDeviceInfo;
use adb_client::server::ADBServer;
use adb_client::ADBDeviceExt;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

static SIZE_TOKEN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([0-9]+(?:\.[0-9]+)?)([kmgtp]?i?b?)?$").expect("invalid size token regex")
});
static PACKAGE_VERSION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"versionCode:([0-9]+)").expect("invalid package regex"));

const STORAGE_MOUNT_PREFERENCE: [&str; 4] =
    ["/data", "/storage/emulated", "/sdcard", "/data/media"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbResult {
    pub stdout: String,
    pub stderr: String,
    pub returncode: i32,
}

impl AdbResult {
    pub fn success(&self) -> bool {
        self.returncode == 0
    }

    pub fn output(&self) -> String {
        self.stdout.trim().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct AdbService {
    server_addr: SocketAddrV4,
    device_serial: Option<String>,
}

impl Default for AdbService {
    fn default() -> Self {
        Self::new()
    }
}

impl AdbService {
    pub fn new() -> Self {
        Self {
            server_addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037),
            device_serial: None,
        }
    }

    pub fn with_server_addr(server_addr: SocketAddrV4) -> Self {
        Self {
            server_addr,
            device_serial: None,
        }
    }

    pub fn device_serial(&self) -> Option<&str> {
        self.device_serial.as_deref()
    }

    pub fn set_device_serial(&mut self, serial: Option<String>) {
        self.device_serial = serial;
    }

    pub async fn start_server(&self) -> Result<AdbResult> {
        tokio::task::spawn_blocking(move || {
            let output = Command::new(crate::services::binary_paths::adb())
                .arg("start-server")
                .output()
                .context("failed to execute `adb start-server`")?;

            Ok(AdbResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                returncode: output.status.code().unwrap_or(0),
            })
        })
        .await
        .context("failed to join start_server task")?
    }

    pub async fn kill_server(&self) -> Result<AdbResult> {
        let server_addr = self.server_addr;
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            server.kill()?;
            Ok(AdbResult {
                stdout: "ADB server killed".to_string(),
                stderr: String::new(),
                returncode: 0,
            })
        })
        .await
        .context("failed to join kill_server task")?
    }

    pub async fn get_devices(&self) -> Result<Vec<RawDeviceInfo>> {
        let server_addr = self.server_addr;
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            let devices = server.devices_long()?;
            Ok(devices
                .into_iter()
                .map(|device| RawDeviceInfo {
                    serial: device.identifier,
                    state: device.state.to_string(),
                    model: if device.model == "Unk" {
                        String::new()
                    } else {
                        device.model
                    },
                    product: if device.product == "Unk" {
                        String::new()
                    } else {
                        device.product
                    },
                })
                .collect())
        })
        .await
        .context("failed to join get_devices task")?
    }

    pub async fn connect_wireless(&self, ip_port: &str) -> Result<AdbResult> {
        let endpoint = ip_port
            .parse::<SocketAddrV4>()
            .with_context(|| format!("invalid wireless endpoint `{ip_port}`"))?;
        let server_addr = self.server_addr;
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            server.connect_device(endpoint)?;
            Ok(AdbResult {
                stdout: format!("connected to {endpoint}"),
                stderr: String::new(),
                returncode: 0,
            })
        })
        .await
        .context("failed to join connect_wireless task")?
    }

    pub async fn disconnect_wireless(&self, ip_port: Option<&str>) -> Result<AdbResult> {
        let server_addr = self.server_addr;
        let endpoint = ip_port.map(|value| value.parse::<SocketAddrV4>());
        tokio::task::spawn_blocking(move || -> Result<AdbResult> {
            if let Some(parsed) = endpoint {
                let mut server = ADBServer::new(server_addr);
                let addr = parsed.context("invalid wireless endpoint for disconnect")?;
                server.disconnect_device(addr)?;
                return Ok(AdbResult {
                    stdout: format!("disconnected {addr}"),
                    stderr: String::new(),
                    returncode: 0,
                });
            }
            let output = Command::new(crate::services::binary_paths::adb())
                .arg("disconnect")
                .output()
                .context("failed to execute `adb disconnect`")?;
            Ok(AdbResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                returncode: output.status.code().unwrap_or(1),
            })
        })
        .await
        .context("failed to join disconnect_wireless task")?
    }

    pub async fn shell(&self, command: &str, serial: Option<&str>) -> Result<AdbResult> {
        let server_addr = self.server_addr;
        let serial = self.resolve_serial(serial);
        let command = command.to_string();
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            let mut device = Self::resolve_device(&mut server, serial.as_deref())?;
            let mut stdout = Vec::<u8>::new();
            let mut stderr = Vec::<u8>::new();
            let exit_code = device.shell_command(&command, Some(&mut stdout), Some(&mut stderr))?;
            Ok(AdbResult {
                stdout: String::from_utf8_lossy(&stdout).to_string(),
                stderr: String::from_utf8_lossy(&stderr).to_string(),
                returncode: i32::from(exit_code.unwrap_or(0)),
            })
        })
        .await
        .context("failed to join shell task")?
    }

    pub async fn install_apk(&self, apk_path: &str, serial: Option<&str>) -> Result<AdbResult> {
        let filename = Path::new(apk_path)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| anyhow!("Invalid APK path"))?;
        let remote_path = format!("/data/local/tmp/{}", filename);
        
        crate::logger::log(&format!("[ADB] install_apk: local='{}', remote='{}', serial='{:?}'", apk_path, remote_path, serial));

        // 1. Push using external adb command for reliability with large binary files
        crate::logger::log("[ADB] Pushing APK using external adb...");
        let serial = self.resolve_serial(serial);
        let serial_for_push = serial.clone();
        let apk_path_owned = apk_path.to_string();
        let remote_path_owned = remote_path.clone();
        
        let push_res = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(crate::services::binary_paths::adb());
            if let Some(s) = &serial_for_push {
                cmd.arg("-s").arg(s);
            }
            cmd.arg("push")
                .arg(&apk_path_owned)
                .arg(&remote_path_owned);
            
            let output = cmd.output()
                .context("failed to execute adb push for APK")?;
            
            Ok::<AdbResult, anyhow::Error>(AdbResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                returncode: output.status.code().unwrap_or(1),
            })
        }).await??;
        
        if !push_res.success() {
            crate::logger::log(&format!("[ADB] Push failed: stdout='{}', stderr='{}'", push_res.stdout, push_res.stderr));
            return Ok(push_res);
        }
        crate::logger::log("[ADB] Push successful.");
        
        // 2. Install using adb_client shell method
        crate::logger::log("[ADB] Running pm install...");
        let install_cmd = format!("pm install -r -d -g '{}'", remote_path);
        let install_res = self.shell(&install_cmd, serial.as_deref()).await?;
        
        crate::logger::log(&format!("[ADB] Install result: returncode={}, stdout='{}', stderr='{}'", 
            install_res.returncode, install_res.stdout, install_res.stderr));
        
        // 3. Cleanup
        let _ = self.shell(&format!("rm '{}'", remote_path), serial.as_deref()).await;
        
        Ok(install_res)
    }

    pub async fn push_file(
        &self,
        local_path: &str,
        remote_path: &str,
        serial: Option<&str>,
    ) -> Result<AdbResult> {
        let server_addr = self.server_addr;
        let serial = self.resolve_serial(serial);
        let local_path = local_path.to_string();
        let remote_path = remote_path.to_string();
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            let mut device = Self::resolve_device(&mut server, serial.as_deref())?;
            let mut file = std::fs::File::open(&local_path)
                .with_context(|| format!("failed to open `{local_path}`"))?;
            device.push(&mut file, &remote_path)?;
            Ok(AdbResult {
                stdout: format!("pushed {local_path} -> {remote_path}"),
                stderr: String::new(),
                returncode: 0,
            })
        })
        .await
        .context("failed to join push_file task")?
    }

    /// Push a directory using adb push command (supports recursive directory copy)
    pub async fn push_dir(
        &self,
        local_path: &str,
        remote_path: &str,
        serial: Option<&str>,
    ) -> Result<AdbResult> {
        let serial = self.resolve_serial(serial);
        let local_path = local_path.to_string();
        let remote_path = remote_path.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(crate::services::binary_paths::adb());
            
            if let Some(s) = serial {
                cmd.arg("-s").arg(s);
            }
            
            cmd.arg("push")
                .arg(&local_path)
                .arg(&remote_path);
            
            let output = cmd.output()
                .context("failed to execute adb push command")?;
            
            Ok(AdbResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                returncode: output.status.code().unwrap_or(1),
            })
        })
        .await
        .context("failed to join push_dir task")?
    }

    pub async fn pull_file(
        &self,
        remote_path: &str,
        local_path: &str,
        serial: Option<&str>,
    ) -> Result<AdbResult> {
        let server_addr = self.server_addr;
        let serial = self.resolve_serial(serial);
        let remote_path = remote_path.to_string();
        let local_path = local_path.to_string();
        tokio::task::spawn_blocking(move || {
            let mut server = ADBServer::new(server_addr);
            let mut device = Self::resolve_device(&mut server, serial.as_deref())?;
            let mut file = std::fs::File::create(&local_path)
                .with_context(|| format!("failed to create `{local_path}`"))?;
            device.pull(&remote_path, &mut file)?;
            file.flush()
                .with_context(|| format!("failed to flush `{local_path}`"))?;
            Ok(AdbResult {
                stdout: format!("pulled {remote_path} -> {local_path}"),
                stderr: String::new(),
                returncode: 0,
            })
        })
        .await
        .context("failed to join pull_file task")?
    }

    pub async fn get_storage_info(&self, serial: Option<&str>) -> Result<HashMap<String, i64>> {
        let result = self.shell("df /data", serial).await?;
        let parsed = Self::parse_storage_info_output(&result.stdout);
        let mut map = HashMap::new();
        map.insert("total_mb".to_string(), parsed.total_mb);
        map.insert("used_mb".to_string(), parsed.used_mb);
        map.insert("free_mb".to_string(), parsed.free_mb);
        Ok(map)
    }

    pub async fn get_battery_info(&self, serial: Option<&str>) -> Result<HashMap<String, String>> {
        let result = self.shell("dumpsys battery", serial).await?;
        let parsed = Self::parse_battery_output(&result.stdout);
        let mut map = HashMap::new();
        map.insert(
            "level_percent".to_string(),
            parsed
                .level_percent
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        map.insert("status".to_string(), parsed.status);
        map.insert("is_charging".to_string(), parsed.is_charging.to_string());
        map.insert(
            "temperature_c".to_string(),
            parsed
                .temperature_c
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        Ok(map)
    }

    pub fn parse_devices_output(output: &str) -> Vec<RawDeviceInfo> {
        output
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty()
                    || trimmed.starts_with("List of")
                    || trimmed.starts_with("* daemon")
                    || trimmed.starts_with("adb server")
                {
                    return None;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 2 {
                    return None;
                }
                let mut model = String::new();
                let mut product = String::new();
                for part in &parts[2..] {
                    if let Some(value) = part.strip_prefix("model:") {
                        model = value.to_string();
                    } else if let Some(value) = part.strip_prefix("product:") {
                        product = value.to_string();
                    }
                }
                Some(RawDeviceInfo {
                    serial: parts[0].to_string(),
                    state: parts[1].to_string(),
                    model,
                    product,
                })
            })
            .collect()
    }

    pub fn parse_storage_info_output(output: &str) -> StorageInfo {
        let mut parsed_candidates: Vec<(usize, StorageInfo)> = Vec::new();

        for raw_line in output.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.to_lowercase().starts_with("filesystem") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let mount_point = parts[parts.len() - 1];
            let total_mb = Self::size_token_to_mb(parts[1], true);
            let used_mb = Self::size_token_to_mb(parts[2], true);
            let free_mb = Self::size_token_to_mb(parts[3], true);
            let Some(total_mb) = total_mb else { continue };
            let Some(used_mb) = used_mb else { continue };
            let Some(free_mb) = free_mb else { continue };

            let score = STORAGE_MOUNT_PREFERENCE
                .iter()
                .enumerate()
                .find_map(|(idx, preferred)| {
                    if mount_point == *preferred || mount_point.starts_with(preferred) {
                        Some(STORAGE_MOUNT_PREFERENCE.len() - idx)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            parsed_candidates.push((
                score,
                StorageInfo {
                    total_mb: total_mb.max(0),
                    used_mb: used_mb.max(0),
                    free_mb: free_mb.max(0),
                },
            ));
        }

        parsed_candidates.sort_by(|left, right| right.0.cmp(&left.0));
        parsed_candidates
            .into_iter()
            .map(|(_, info)| info)
            .next()
            .unwrap_or_default()
    }

    pub fn parse_battery_output(output: &str) -> BatteryInfo {
        let mut parsed = HashMap::<String, String>::new();
        for raw_line in output.lines() {
            if let Some((key, value)) = raw_line.split_once(':') {
                parsed.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        let level_percent = parsed.get("level").and_then(|level| {
            let level_value = level.parse::<f64>().ok()?;
            let scale_value = parsed
                .get("scale")
                .and_then(|scale| scale.parse::<f64>().ok())
                .unwrap_or(100.0);
            if scale_value <= 0.0 {
                None
            } else {
                Some(((level_value / scale_value) * 100.0).round() as i64)
            }
        });

        let status_code = parsed
            .get("status")
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(1);
        let status = match status_code {
            2 => "charging",
            3 => "discharging",
            4 => "not_charging",
            5 => "full",
            _ => "unknown",
        }
        .to_string();

        let temperature_c = parsed
            .get("temperature")
            .and_then(|value| value.parse::<f64>().ok())
            .map(|value| ((value / 10.0) * 10.0).round() / 10.0);

        BatteryInfo {
            level_percent,
            status: status.clone(),
            is_charging: status == "charging" || status == "full",
            temperature_c,
        }
    }

    pub fn parse_packages_with_versions_output(output: &str) -> HashMap<String, Option<String>> {
        let mut parsed = HashMap::<String, Option<String>>::new();
        for raw_line in output.lines() {
            let line = raw_line.trim();
            if !line.starts_with("package:") {
                continue;
            }

            let payload = line["package:".len()..].trim();
            if payload.is_empty() {
                continue;
            }

            let package_name = if let Some((_, rhs)) = payload.split_once('=') {
                rhs.split_whitespace().next().unwrap_or_default()
            } else {
                payload.split_whitespace().next().unwrap_or_default()
            };
            if package_name.is_empty() {
                continue;
            }

            let version_code = PACKAGE_VERSION_PATTERN
                .captures(line)
                .and_then(|captures| captures.get(1))
                .map(|m| m.as_str().to_string());

            parsed.insert(package_name.to_string(), version_code);
        }
        parsed
    }

    fn resolve_serial(&self, serial: Option<&str>) -> Option<String> {
        serial
            .map(ToString::to_string)
            .or_else(|| self.device_serial.clone())
    }

    fn resolve_device(
        server: &mut ADBServer,
        serial: Option<&str>,
    ) -> Result<adb_client::server_device::ADBServerDevice> {
        let mut result = if let Some(serial) = serial {
            server.get_device_by_name(serial)
        } else {
            server.get_device()
        };

        if result.is_err() {
            // Try starting server and retry once
            let _ = Command::new(crate::services::binary_paths::adb()).arg("start-server").output();
            result = if let Some(serial) = serial {
                server.get_device_by_name(serial)
            } else {
                server.get_device()
            };
        }

        match result {
            Ok(device) => Ok(device),
            Err(err) => {
                if let Some(serial) = serial {
                    Err(anyhow!("failed to resolve adb device `{serial}`: {err}"))
                } else {
                    Err(anyhow!("failed to resolve adb device: {err}"))
                }
            }
        }
    }

    fn size_token_to_mb(token: &str, assume_kib_without_suffix: bool) -> Option<i64> {
        let cleaned = token.trim().to_lowercase();
        if cleaned.is_empty() {
            return None;
        }

        let captures = SIZE_TOKEN_PATTERN.captures(&cleaned)?;
        let raw_value = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let suffix = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        let mb = match suffix {
            "" | "b" => {
                if assume_kib_without_suffix {
                    raw_value / 1024.0
                } else {
                    raw_value / (1024.0 * 1024.0)
                }
            }
            "k" | "kb" | "ki" | "kib" => raw_value / 1024.0,
            "m" | "mb" | "mi" | "mib" => raw_value,
            "g" | "gb" | "gi" | "gib" => raw_value * 1024.0,
            "t" | "tb" | "ti" | "tib" => raw_value * 1024.0 * 1024.0,
            "p" | "pb" | "pi" | "pib" => raw_value * 1024.0 * 1024.0 * 1024.0,
            _ => return None,
        };

        Some(mb as i64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StorageInfo {
    pub total_mb: i64,
    pub used_mb: i64,
    pub free_mb: i64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BatteryInfo {
    pub level_percent: Option<i64>,
    pub status: String,
    pub is_charging: bool,
    pub temperature_c: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devices_output() {
        let output = "List of devices attached
1WMHH824D50421\tdevice product:hollywood model:Quest_3 device:eureka transport_id:2
192.168.1.10:5555\toffline transport_id:7";

        let parsed = AdbService::parse_devices_output(output);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].serial, "1WMHH824D50421");
        assert_eq!(parsed[0].state, "device");
        assert_eq!(parsed[0].model, "Quest_3");
        assert_eq!(parsed[0].product, "hollywood");
        assert_eq!(parsed[1].serial, "192.168.1.10:5555");
        assert_eq!(parsed[1].state, "offline");
    }

    #[test]
    fn test_parse_storage_info_output_prefers_data_mount() {
        let output = "Filesystem     1K-blocks      Used Available Use% Mounted on
/dev/fuse       120000000  30000000 90000000  25% /storage/emulated
/dev/block/dm-5  64000000  16000000 48000000  25% /data";
        let parsed = AdbService::parse_storage_info_output(output);
        assert_eq!(
            parsed,
            StorageInfo {
                total_mb: 62500,
                used_mb: 15625,
                free_mb: 46875
            }
        );
    }

    #[test]
    fn test_parse_battery_output() {
        let output = "Current Battery Service state:
  AC powered: false
  USB powered: true
  Wireless powered: false
  status: 2
  level: 71
  scale: 100
  temperature: 318";

        let parsed = AdbService::parse_battery_output(output);
        assert_eq!(parsed.level_percent, Some(71));
        assert_eq!(parsed.status, "charging");
        assert!(parsed.is_charging);
        assert_eq!(parsed.temperature_c, Some(31.8));
    }

    #[test]
    fn test_parse_packages_with_versions_output() {
        let output = "package:com.test.one versionCode:12345
package:com.test.two versionCode:8
package:com.test.three";
        let parsed = AdbService::parse_packages_with_versions_output(output);
        assert_eq!(parsed.get("com.test.one"), Some(&Some("12345".to_string())));
        assert_eq!(parsed.get("com.test.two"), Some(&Some("8".to_string())));
        assert_eq!(parsed.get("com.test.three"), Some(&None));
    }
}
