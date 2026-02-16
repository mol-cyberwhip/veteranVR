use crate::logger;
use crate::models::config::PublicConfig;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::Path;
use std::process::Stdio;
use std::sync::RwLock;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DownloadProgress {
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub percent: f64,
    pub speed: String,
    pub eta: String,
}

impl DownloadProgress {
    pub fn speed_display(&self) -> &str {
        if self.speed.is_empty() {
            "calculating..."
        } else {
            &self.speed
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcloneResult {
    pub stdout: String,
    pub stderr: String,
    pub returncode: i32,
}

impl RcloneResult {
    pub fn success(&self) -> bool {
        self.returncode == 0
    }
}

#[derive(Debug)]
pub struct RcloneService {
    rclone_path: String,
    password: RwLock<String>,
    base_uri: RwLock<String>,
    daemon_process: Mutex<Option<Child>>,
    rc_port: RwLock<Option<u16>>,
    http_client: reqwest::Client,
    active_jobs: Mutex<HashMap<String, u64>>,
}

impl Default for RcloneService {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RcloneService {
    pub fn new(rclone_path: Option<String>) -> Self {
        Self {
            rclone_path: rclone_path.unwrap_or_else(|| "rclone".to_string()),
            password: RwLock::new(String::new()),
            base_uri: RwLock::new(String::new()),
            daemon_process: Mutex::new(None),
            rc_port: RwLock::new(None),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            active_jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_public_config(&self, config: &PublicConfig) {
        let mut base_uri = self.base_uri.write().unwrap();
        let mut password = self.password.write().unwrap();
        *base_uri = config.base_uri.clone();
        *password = config.password.clone();
    }

    pub fn base_uri(&self) -> String {
        self.base_uri.read().unwrap().clone()
    }

    fn find_free_port() -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .context("Failed to bind to find free port")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }

    async fn rc_health_check(&self, port: u16) -> Result<()> {
        let url = format!("http://127.0.0.1:{}/core/version", port);
        let response = self.http_client.post(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Health check failed with status: {}", response.status()))
        }
    }

    async fn rc_post(&self, port: u16, endpoint: &str, body: Value) -> Result<Value> {
        let url = format!("http://127.0.0.1:{}/{}", port, endpoint);
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context(format!("Failed to POST to {}", endpoint))?;
        
        let status = response.status();
        let json: Value = response.json().await.context("Failed to parse JSON response")?;
        
        if status.is_success() {
            Ok(json)
        } else {
            Err(anyhow::anyhow!("RC API error: {:?}", json))
        }
    }

    pub async fn ensure_daemon(&self) -> Result<u16> {
        // Fast path: check if daemon is already running
        let port_fast = *self.rc_port.read().unwrap();
        if let Some(port) = port_fast {
            if self.rc_health_check(port).await.is_ok() {
                logger::log(&format!("[RCLONE] Using existing daemon on port {}", port));
                return Ok(port);
            }
        }

        logger::log("[RCLONE] Starting rclone daemon...");

        // Slow path: start the daemon
        let mut daemon_lock = self.daemon_process.lock().await;
        
        // Double-check after acquiring lock
        let port_check = *self.rc_port.read().unwrap();
        if let Some(port) = port_check {
            if self.rc_health_check(port).await.is_ok() {
                logger::log(&format!("[RCLONE] Daemon already running on port {}", port));
                return Ok(port);
            }
        }

        // Kill any existing daemon process
        if let Some(mut child) = daemon_lock.take() {
            logger::log("[RCLONE] Killing existing daemon process");
            let _ = child.kill().await;
        }

        // Find a free port and start the daemon
        let port = Self::find_free_port()?;
        logger::log(&format!("[RCLONE] Found free port: {}", port));
        
        let password = self.password.read().unwrap().clone();

        let mut cmd = Command::new(&self.rclone_path);
        cmd.args([
            "rcd",
            "--rc-no-auth",
            &format!("--rc-addr=127.0.0.1:{}", port),
            "--ask-password=false",
            "--config",
            "/dev/null",
            "--tpslimit",
            "1.0",
            "--tpslimit-burst",
            "3",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        if !password.is_empty() {
            cmd.env("RCLONE_CONFIG_PASS", password);
            logger::log("[RCLONE] Using password from config");
        }

        let child = cmd.spawn().context("Failed to spawn rclone daemon")?;
        logger::log(&format!("[RCLONE] Daemon spawned with PID: {:?}", child.id()));
        *daemon_lock = Some(child);
        drop(daemon_lock);

        // Poll health check up to 20 times at 100ms intervals
        for i in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if self.rc_health_check(port).await.is_ok() {
                logger::log(&format!("[RCLONE] Daemon healthy after {} checks", i + 1));
                *self.rc_port.write().unwrap() = Some(port);
                
                // Create the remote config
                if let Err(e) = self.create_remote_config(port).await {
                    logger::log(&format!("[RCLONE] WARNING: Failed to create remote config: {}", e));
                }
                
                return Ok(port);
            }
        }

        logger::log("[RCLONE] ERROR: Daemon failed to start within 2 seconds");
        Err(anyhow::anyhow!("Rclone daemon failed to start within 2 seconds"))
    }

    async fn create_remote_config(&self, port: u16) -> Result<()> {
        let base_uri = self.base_uri.read().unwrap().clone();
        let base_uri_trimmed = base_uri.trim_end_matches('/');
        
        logger::log(&format!("[RCLONE] Creating remote config 'vrp' with URL: {}", base_uri_trimmed));
        
        let body = serde_json::json!({
            "name": "vrp",
            "type": "http",
            "parameters": {
                "url": base_uri_trimmed
            }
        });
        
        let response = self.rc_post(port, "config/create", body).await?;
        logger::log(&format!("[RCLONE] Config created: {}", response));
        
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Try graceful shutdown via RC API
        let port = *self.rc_port.read().unwrap();
        if let Some(port) = port {
            let _ = self.rc_post(port, "core/quit", serde_json::json!({})).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        // Force kill if still running
        let mut daemon_lock = self.daemon_process.lock().await;
        if let Some(mut child) = daemon_lock.take() {
            let _ = child.kill().await;
        }

        *self.rc_port.write().unwrap() = None;
        Ok(())
    }

    pub async fn sync_metadata(&self, download_dir: &Path) -> Result<RcloneResult> {
        let port = self.ensure_daemon().await?;
        
        // Use the named remote "vrp" which was configured in ensure_daemon
        let src_fs = "vrp:meta.7z".to_string();
        logger::log(&format!("[RCLONE] sync_metadata from '{}'", src_fs));

        let body = serde_json::json!({
            "srcFs": src_fs,
            "dstFs": download_dir.to_string_lossy().to_string(),
            "_config": {
                "Inplace": true,
                "SizeOnly": true
            }
        });

        let response = self.rc_post(port, "sync/sync", body).await?;

        // Check for errors in the response
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Ok(RcloneResult {
                    stdout: String::new(),
                    stderr: error.to_string(),
                    returncode: 1,
                });
            }
        }

        Ok(RcloneResult {
            stdout: response.to_string(),
            stderr: String::new(),
            returncode: 0,
        })
    }

    pub async fn download_game(
        &self,
        game_hash: &str,
        download_dir: &Path,
        bandwidth_limit_mbps: f64,
        progress_sender: Option<tokio::sync::mpsc::UnboundedSender<DownloadProgress>>,
    ) -> Result<RcloneResult> {
        logger::log(&format!("[RCLONE] Starting download for game_hash: {}", game_hash));
        
        let port = self.ensure_daemon().await?;
        let base_uri = self.base_uri.read().unwrap().clone();
        
        logger::log(&format!("[RCLONE] base_uri: {}, port: {}", base_uri, port));

        tokio::fs::create_dir_all(download_dir)
            .await
            .with_context(|| format!("failed to create {}", download_dir.display()))?;
        
        logger::log(&format!("[RCLONE] Download dir created: {}", download_dir.display()));

        // Set initial bandwidth limit
        self.set_bandwidth_limit_internal(port, bandwidth_limit_mbps).await?;
        logger::log(&format!("[RCLONE] Bandwidth limit set: {} Mbps", bandwidth_limit_mbps));

        // Start async copy job
        // Use the named remote "vrp" which was configured in ensure_daemon
        // The format is "vrp:<path>" where path is the game_hash directory
        let src_fs = format!("vrp:{}/", game_hash);
        let dst_fs = download_dir.to_string_lossy().to_string();
        logger::log(&format!("[RCLONE] Starting sync/copy from '{}' to '{}'", src_fs, dst_fs));
        
        let body = serde_json::json!({
            "srcFs": src_fs,
            "dstFs": dst_fs,
            "_async": true,
            "_config": {
                "Inplace": true
            }
        });

        let response = match self.rc_post(port, "sync/copy", body).await {
            Ok(resp) => {
                logger::log(&format!("[RCLONE] sync/copy response: {}", resp));
                resp
            }
            Err(e) => {
                logger::log(&format!("[RCLONE] ERROR in sync/copy: {}", e));
                return Err(e);
            }
        };
        
        let job_id = response
            .get("jobid")
            .and_then(|v| v.as_u64())
            .context("Missing jobid in response")?;
        
        logger::log(&format!("[RCLONE] Job started with ID: {}", job_id));

        // Store job ID
        self.active_jobs.lock().await.insert(game_hash.to_string(), job_id);

        // Poll loop
        let result = loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Check job status
            let status_body = serde_json::json!({ "jobid": job_id });
            let status = match self.rc_post(port, "job/status", status_body).await {
                Ok(s) => s,
                Err(e) => {
                    logger::log(&format!("[RCLONE] ERROR getting job status: {}", e));
                    break Err(e);
                }
            };

            // Get stats for progress
            let stats_body = serde_json::json!({ "group": format!("job/{}", job_id) });
            let stats = self.rc_post(port, "core/stats", stats_body).await.ok();

            if let Some(ref s) = stats {
                if let Some(progress) = Self::parse_rc_stats(s) {
                    if let Some(sender) = progress_sender.as_ref() {
                        let _ = sender.send(progress);
                    }
                }
            }

            // Check if finished
            if status.get("finished").and_then(|v| v.as_bool()).unwrap_or(false) {
                let success = status.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                let error = status.get("error").and_then(|v| v.as_str()).unwrap_or("");

                if success {
                    logger::log(&format!("[RCLONE] Job {} completed successfully", job_id));
                    break Ok(RcloneResult {
                        stdout: status.to_string(),
                        stderr: String::new(),
                        returncode: 0,
                    });
                } else {
                    logger::log(&format!("[RCLONE] Job {} failed: {}", job_id, error));
                    break Ok(RcloneResult {
                        stdout: String::new(),
                        stderr: error.to_string(),
                        returncode: 1,
                    });
                }
            }
        };

        // Remove job from active jobs
        self.active_jobs.lock().await.remove(game_hash);

        result
    }

    fn parse_rc_stats(stats: &Value) -> Option<DownloadProgress> {
        let bytes = stats.get("bytes")?.as_i64()?;
        let total_bytes = stats.get("totalBytes")?.as_i64()?;
        let speed = stats.get("speed")?.as_f64()?;
        let eta = stats.get("eta")?.as_i64()?;

        let percent = if total_bytes > 0 {
            (bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        Some(DownloadProgress {
            bytes_transferred: bytes,
            total_bytes,
            percent,
            speed: Self::format_speed(speed),
            eta: Self::format_eta(eta),
        })
    }

    fn format_speed(bytes_per_sec: f64) -> String {
        if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} GiB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
        } else if bytes_per_sec >= 1024.0 * 1024.0 {
            format!("{:.1} MiB/s", bytes_per_sec / (1024.0 * 1024.0))
        } else if bytes_per_sec >= 1024.0 {
            format!("{:.1} KiB/s", bytes_per_sec / 1024.0)
        } else {
            format!("{:.0} B/s", bytes_per_sec)
        }
    }

    fn format_eta(seconds: i64) -> String {
        if seconds < 0 {
            return "calculating...".to_string();
        }
        if seconds < 60 {
            return format!("{}s", seconds);
        }
        if seconds < 3600 {
            return format!("{}m{}s", seconds / 60, seconds % 60);
        }
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h{}m", hours, minutes)
    }

    pub async fn cancel_download(&self) -> Result<()> {
        let jobs = self.active_jobs.lock().await.clone();
        let port = self.ensure_daemon().await?;

        for (_, job_id) in jobs {
            let body = serde_json::json!({ "jobid": job_id });
            let _ = self.rc_post(port, "job/stop", body).await;
        }

        Ok(())
    }

    async fn set_bandwidth_limit_internal(&self, port: u16, mbps: f64) -> Result<()> {
        let rate = if mbps <= 0.0 {
            "off".to_string()
        } else {
            format!("{:.1}M", mbps)
        };

        let body = serde_json::json!({ "rate": rate });
        self.rc_post(port, "core/bwlimit", body).await?;
        Ok(())
    }

    pub async fn pause_downloads(&self) -> Result<()> {
        let port = self.ensure_daemon().await?;
        let body = serde_json::json!({ "rate": "0" });
        self.rc_post(port, "core/bwlimit", body).await?;
        Ok(())
    }

    pub async fn resume_downloads(&self, bandwidth_limit_mbps: f64) -> Result<()> {
        let port = self.ensure_daemon().await?;
        self.set_bandwidth_limit_internal(port, bandwidth_limit_mbps).await
    }

    pub async fn set_bandwidth_limit(&self, mbps: f64) -> Result<()> {
        let port = self.ensure_daemon().await?;
        self.set_bandwidth_limit_internal(port, mbps).await
    }

    pub fn parse_bytes(size_str: &str) -> i64 {
        let normalized = size_str.trim().to_uppercase();
        let parse_value = |suffix: &str, scale: f64| -> Option<i64> {
            let value = normalized.strip_suffix(suffix)?.trim().parse::<f64>().ok()?;
            Some((value * scale) as i64)
        };

        if let Some(value) = parse_value("PIB", 1024_f64.powi(5)) {
            return value;
        }
        if let Some(value) = parse_value("TIB", 1024_f64.powi(4)) {
            return value;
        }
        if let Some(value) = parse_value("GIB", 1024_f64.powi(3)) {
            return value;
        }
        if let Some(value) = parse_value("MIB", 1024_f64.powi(2)) {
            return value;
        }
        if let Some(value) = parse_value("KIB", 1024_f64) {
            return value;
        }

        if let Some(value) = parse_value("PB", 1000_f64.powi(5)) {
            return value;
        }
        if let Some(value) = parse_value("TB", 1000_f64.powi(4)) {
            return value;
        }
        if let Some(value) = parse_value("GB", 1000_f64.powi(3)) {
            return value;
        }
        if let Some(value) = parse_value("MB", 1000_f64.powi(2)) {
            return value;
        }
        if let Some(value) = parse_value("KB", 1000_f64) {
            return value;
        }

        if let Some(value) = parse_value("B", 1.0) {
            return value;
        }

        normalized.parse::<f64>().map(|value| value as i64).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bytes_supports_binary_units() {
        assert_eq!(RcloneService::parse_bytes("1 KiB"), 1024);
        assert_eq!(RcloneService::parse_bytes("1.5 MiB"), 1_572_864);
        assert_eq!(RcloneService::parse_bytes("2 GiB"), 2_147_483_648);
    }

    #[test]
    fn parse_bytes_supports_decimal_units() {
        assert_eq!(RcloneService::parse_bytes("1 KB"), 1000);
        assert_eq!(RcloneService::parse_bytes("2.5 MB"), 2_500_000);
        assert_eq!(RcloneService::parse_bytes("3 GB"), 3_000_000_000);
    }

    #[test]
    fn parse_rc_stats_extracts_fields() {
        let stats = serde_json::json!({
            "bytes": 104_857_600,
            "totalBytes": 524_288_000,
            "speed": 11_010_048.0,
            "eta": 40
        });
        
        let progress = RcloneService::parse_rc_stats(&stats).expect("must parse stats");
        assert_eq!(progress.bytes_transferred, 104_857_600);
        assert_eq!(progress.total_bytes, 524_288_000);
        assert_eq!(progress.percent, 20.0);
        assert_eq!(progress.speed, "10.5 MiB/s");
        assert_eq!(progress.eta, "40s");
    }

    #[test]
    fn format_speed_formats_correctly() {
        assert_eq!(RcloneService::format_speed(100.0), "100 B/s");
        assert_eq!(RcloneService::format_speed(1024.0), "1.0 KiB/s");
        assert_eq!(RcloneService::format_speed(1024.0 * 1024.0), "1.0 MiB/s");
        assert_eq!(RcloneService::format_speed(1024.0 * 1024.0 * 1024.0), "1.0 GiB/s");
    }

    #[test]
    fn format_eta_formats_correctly() {
        assert_eq!(RcloneService::format_eta(30), "30s");
        assert_eq!(RcloneService::format_eta(90), "1m30s");
        assert_eq!(RcloneService::format_eta(3660), "1h1m");
        assert_eq!(RcloneService::format_eta(-1), "calculating...");
    }
}
