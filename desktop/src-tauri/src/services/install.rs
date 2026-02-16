use crate::services::adb::AdbService;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct InstallService {
    adb: AdbService,
    /// Tracks package names currently being installed to prevent concurrent installs.
    installing: Arc<Mutex<HashSet<String>>>,
}

impl InstallService {
    pub fn new(adb: AdbService) -> Self {
        Self {
            adb,
            installing: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Try to acquire the install lock for a package. Returns false if already installing.
    pub async fn try_start_install(&self, package_name: &str) -> bool {
        self.installing.lock().await.insert(package_name.to_string())
    }

    /// Release the install lock for a package.
    pub async fn finish_install(&self, package_name: &str) {
        self.installing.lock().await.remove(package_name);
    }

    /// Install a game following the same steps as the Windows sideloader:
    ///
    /// 1. Extract `{hash_dir}/{hash}.7z.001` → `{download_dir}/` (parent)
    /// 2. Delete `{hash_dir}/` (the archive directory)
    /// 3. Find the extracted game folder `{download_dir}/{release_name}/`
    /// 4. Check for install.txt → if found, run custom install and STOP
    /// 5. Find the first .apk in the game folder → install it
    /// 6. Find OBB dir `{game_folder}/{package_name}/` → push to device
    pub async fn install_game(
        &self,
        hash_dir: &Path,
        package_name: &str,
        release_name: &str,
        serial: Option<&str>,
        password: Option<String>,
        status_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<InstallResult> {
        crate::logger::log(&format!(
            "[INSTALL] install_game: hash_dir='{}', package='{}', release='{}'",
            hash_dir.display(), package_name, release_name
        ));

        let download_dir = hash_dir.parent().unwrap_or(hash_dir);

        if !hash_dir.exists() {
            // The hash directory may have been deleted after a previous extraction.
            // Check if the extracted release_name folder already exists in the download dir.
            let release_dir = download_dir.join(release_name);
            if release_dir.exists() {
                crate::logger::log(&format!(
                    "[INSTALL] Hash dir missing but found extracted folder: {}",
                    release_dir.display()
                ));
                return self.install_from_game_dir(&release_dir, package_name, serial, password, status_sender).await;
            }
            return Ok(InstallResult {
                success: false,
                message: format!("Game directory not found: {}", hash_dir.display()),
            });
        }

        // Step 1: Extract archives into the parent download directory
        // (matches Windows: `7z x {hash}.7z.001 -o{DownloadDir}`)
        if let Some(sender) = status_sender.as_ref() {
            let _ = sender.send("Extracting archives...".to_string());
        }

        let archives = find_archives(hash_dir)?;
        crate::logger::log(&format!("[INSTALL] Found {} archives to extract", archives.len()));

        for archive in &archives {
            crate::logger::log(&format!("[INSTALL] Extracting: {} → {}", archive.display(), download_dir.display()));
            if let Err(e) = crate::services::extract::ExtractService::extract_7z(archive, download_dir, password.as_deref()) {
                crate::logger::log(&format!("[INSTALL] Extraction failed: {:?}", e));
                return Err(e);
            }
        }

        // Step 2: Delete the hash directory (the .7z archive files)
        // (matches Windows: `FileSystemUtilities.TryDeleteDirectory({hash_dir})`)
        if !archives.is_empty() {
            crate::logger::log(&format!("[INSTALL] Deleting archive directory: {}", hash_dir.display()));
            if let Err(e) = std::fs::remove_dir_all(hash_dir) {
                crate::logger::log(&format!("[INSTALL] Warning: failed to delete archive dir: {}", e));
                // Non-fatal — continue with install
            }
        }

        // Step 3: Find the extracted game folder
        // (matches Windows: `{DownloadDir}\{gameName}\`)
        let game_dir = download_dir.join(release_name);
        if !game_dir.exists() {
            // Fallback: if hash_dir still exists (archives were empty / already extracted),
            // use it directly as the game dir
            if hash_dir.exists() {
                return self.install_from_game_dir(hash_dir, package_name, serial, password, status_sender).await;
            }
            return Ok(InstallResult {
                success: false,
                message: format!(
                    "Extracted game folder not found: {}. Expected release name directory after extraction.",
                    game_dir.display()
                ),
            });
        }

        self.install_from_game_dir(&game_dir, package_name, serial, password, status_sender).await
    }

    /// Install from a game directory that contains the APK, OBB, and optionally install.txt.
    /// This matches the Windows sideloader's install logic after extraction.
    async fn install_from_game_dir(
        &self,
        game_dir: &Path,
        package_name: &str,
        serial: Option<&str>,
        password: Option<String>,
        status_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<InstallResult> {
        crate::logger::log(&format!("[INSTALL] Installing from game dir: {}", game_dir.display()));

        // Step 4: Check for install.txt
        // (matches Windows: checks for install.txt / Install.txt)
        let install_txt = find_install_txt(game_dir);
        if let Some(install_txt_path) = install_txt {
            if let Some(sender) = status_sender.as_ref() {
                let _ = sender.send("Running custom install commands...".to_string());
            }
            crate::logger::log(&format!("[INSTALL] Found install.txt: {}", install_txt_path.display()));

            // Extract any nested .7z files first (matches Windows Sideloader.cs)
            let nested_archives = find_archives(install_txt_path.parent().unwrap_or(game_dir))?;
            for archive in &nested_archives {
                crate::logger::log(&format!("[INSTALL] Extracting nested archive: {}", archive.display()));
                let extract_dir = archive.parent().unwrap_or(game_dir);
                let _ = crate::services::extract::ExtractService::extract_7z(archive, extract_dir, password.as_deref());
            }

            let (success, message) = self
                .execute_install_txt(&install_txt_path, game_dir, serial)
                .await
                .context("failed to execute install.txt")?;

            // Windows sideloader STOPS after install.txt — does not continue with APK/OBB
            return Ok(InstallResult { success, message });
        }

        let mut results = Vec::new();

        // Step 5: Find and install the APK
        // (matches Windows: `Directory.GetFiles(gameName).FirstOrDefault(f => ext == ".apk")`)
        let apk = find_first_apk(game_dir);
        match apk {
            Some(apk_path) => {
                if let Some(sender) = status_sender.as_ref() {
                    let _ = sender.send(format!("Installing {}...", apk_path.file_name().unwrap_or_default().to_string_lossy()));
                }
                crate::logger::log(&format!("[INSTALL] Installing APK: {}", apk_path.display()));

                let install_result = self
                    .adb
                    .install_apk(&apk_path.to_string_lossy(), serial)
                    .await
                    .with_context(|| format!("failed to install {}", apk_path.display()))?;

                let has_error = !install_result.stderr.is_empty()
                    && (install_result.stderr.contains("Error")
                        || install_result.stderr.contains("Exception")
                        || install_result.stderr.contains("Failed"));
                let explicit_success = install_result.output().contains("Success");

                if (install_result.success() && !has_error) || explicit_success {
                    crate::logger::log(&format!("[INSTALL] APK installed successfully"));
                    results.push("APK installed successfully".to_string());
                } else {
                    let error_msg = if install_result.stderr.is_empty() {
                        install_result.output()
                    } else {
                        install_result.stderr.clone()
                    };
                    crate::logger::log(&format!("[INSTALL] APK install failed: {}", error_msg));

                    // Try reinstall on eligible errors
                    let is_reinstall_eligible = error_msg.contains("signatures do not match")
                        || error_msg.contains("INSTALL_FAILED_VERSION_DOWNGRADE")
                        || error_msg.contains("failed to install")
                        || error_msg.contains("INSUFFICIENT_STORAGE");

                    if is_reinstall_eligible {
                        if let Some(sender) = status_sender.as_ref() {
                            let _ = sender.send("Attempting reinstall with backup...".to_string());
                        }
                        match self.reinstall_with_backup(&apk_path, package_name, serial, status_sender.clone()).await {
                            Ok(r) if r.success => {
                                results.push("Reinstalled successfully".to_string());
                            }
                            Ok(r) => {
                                return Ok(InstallResult {
                                    success: false,
                                    message: format!("Reinstall failed: {}", r.message),
                                });
                            }
                            Err(e) => {
                                return Ok(InstallResult {
                                    success: false,
                                    message: format!("Reinstall error: {}", e),
                                });
                            }
                        }
                    } else {
                        return Ok(InstallResult {
                            success: false,
                            message: format!("APK install failed: {}", error_msg),
                        });
                    }
                }
            }
            None => {
                crate::logger::log("[INSTALL] No APK found in game directory");
                return Ok(InstallResult {
                    success: false,
                    message: format!("No APK found in {}", game_dir.display()),
                });
            }
        }

        // Step 6: Find and push OBB
        // (matches Windows: checks for `{game_dir}/{package_name}/` directory)
        let obb_dir = game_dir.join(package_name);
        if obb_dir.is_dir() {
            if let Some(sender) = status_sender.as_ref() {
                let _ = sender.send(format!("Copying OBB for {}...", package_name));
            }

            // Delete old OBB on device first (matches Windows deleteOBB())
            let remote_obb = format!("/sdcard/Android/obb/{}", package_name);
            crate::logger::log(&format!("[INSTALL] Deleting old OBB: {}", remote_obb));
            let _ = self.adb.shell(&format!("rm -rf \"{}\"", remote_obb), serial).await;

            // Create remote directory
            let _ = self.adb.shell(&format!("mkdir -p \"{}\"", remote_obb), serial).await;

            // Push OBB directory
            crate::logger::log(&format!("[INSTALL] Pushing OBB: {} → {}", obb_dir.display(), remote_obb));
            let pushed = self
                .adb
                .push_dir(&obb_dir.to_string_lossy(), "/sdcard/Android/obb/", serial)
                .await?;

            if pushed.success() {
                crate::logger::log("[INSTALL] OBB pushed successfully");
                results.push(format!("OBB {}: Success", package_name));
            } else {
                let err = if pushed.stderr.is_empty() { pushed.output() } else { pushed.stderr };
                crate::logger::log(&format!("[INSTALL] OBB push failed: {}", err));
                return Ok(InstallResult {
                    success: false,
                    message: format!("OBB push failed: {}", err),
                });
            }
        } else {
            crate::logger::log(&format!("[INSTALL] No OBB directory found at {}", obb_dir.display()));
        }

        if results.is_empty() {
            return Ok(InstallResult {
                success: false,
                message: "No installable content found".to_string(),
            });
        }

        Ok(InstallResult {
            success: true,
            message: results.join("\n"),
        })
    }

    pub async fn uninstall_game(&self, package_name: &str, serial: Option<&str>) -> Result<InstallResult> {
        crate::logger::log(&format!("[UNINSTALL] Starting uninstall for {}", package_name));
        let uninstall = self
            .adb
            .shell(&format!("pm uninstall {package_name}"), serial)
            .await
            .context("failed to run uninstall")?;

        crate::logger::log(&format!("[UNINSTALL] pm uninstall result: {}", uninstall.output()));

        crate::logger::log(&format!("[UNINSTALL] Cleaning up OBB for {}", package_name));
        let _ = self
            .adb
            .shell(&format!("rm -rf /sdcard/Android/obb/{package_name}"), serial)
            .await;

        crate::logger::log(&format!("[UNINSTALL] Cleaning up data for {}", package_name));
        let _ = self
            .adb
            .shell(&format!("rm -rf /sdcard/Android/data/{package_name}"), serial)
            .await;

        if uninstall.output().contains("Success") {
            crate::logger::log(&format!("[UNINSTALL] {} uninstalled successfully", package_name));
            Ok(InstallResult {
                success: true,
                message: format!("Uninstalled {package_name}"),
            })
        } else {
            crate::logger::log(&format!("[UNINSTALL] {} uninstall failed: {}", package_name, uninstall.output()));
            Ok(InstallResult {
                success: false,
                message: format!("Failed to uninstall {package_name}: {}", uninstall.output()),
            })
        }
    }

    async fn execute_install_txt(
        &self,
        install_txt: &Path,
        game_dir: &Path,
        serial: Option<&str>,
    ) -> Result<(bool, String)> {
        let mut results = Vec::new();

        let contents = std::fs::read_to_string(install_txt)
            .with_context(|| format!("failed reading {}", install_txt.display()))?;

        let work_dir = install_txt.parent().unwrap_or(game_dir);

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }

            crate::logger::log(&format!("[INSTALL] install.txt line: {}", line));

            let Some(argv) = Self::parse_install_txt_line(line) else {
                continue;
            };

            if argv[0] == "install" && argv.len() >= 2 {
                let apk_path = work_dir.join(&argv[1]);
                crate::logger::log(&format!("[INSTALL] install.txt: installing APK {}", apk_path.display()));
                let res = self.adb.install_apk(&apk_path.to_string_lossy(), serial).await?;
                if !res.success() && !res.output().contains("Success") {
                    results.push(format!("Install failed: {}", res.stderr));
                }
                continue;
            }

            if argv[0] == "push" && argv.len() >= 3 {
                let local = work_dir.join(&argv[1]);
                let remote = &argv[2];
                crate::logger::log(&format!("[INSTALL] install.txt: pushing {} to {}", local.display(), remote));
                let res = self.adb.push_file(&local.to_string_lossy(), remote, serial).await?;
                if !res.success() {
                    results.push(format!("Push failed: {}", res.stderr));
                }
                continue;
            }

            if argv[0] == "shell" {
                let shell_command = argv[1..].join(" ");
                crate::logger::log(&format!("[INSTALL] install.txt: shell {}", shell_command));
                let res = self.adb.shell(&shell_command, serial).await?;
                if !res.stderr.trim().is_empty() && !res.stderr.contains("mkdir") {
                    results.push(format!("Warning: {}", res.stderr.trim()));
                }
                continue;
            }

            crate::logger::log(&format!("[INSTALL] install.txt: unsupported command: {}", argv.join(" ")));
        }

        results.push("Custom install successful!".to_string());
        Ok((true, results.join("\n")))
    }

    fn parse_install_txt_line(line: &str) -> Option<Vec<String>> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let command = if trimmed.starts_with("adb") {
             trimmed.strip_prefix("adb")?.trim()
        } else {
             return None;
        };

        if command.is_empty() {
            return None;
        }

        let parts = command
            .split_whitespace()
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if parts.is_empty() {
            return None;
        }

        Some(parts)
    }

    async fn reinstall_with_backup(
        &self,
        apk_path: &Path,
        package_name: &str,
        serial: Option<&str>,
        status_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<InstallResult> {
        if package_name.is_empty() {
            return Ok(InstallResult {
                success: false,
                message: "Cannot reinstall: unable to determine package name".to_string(),
            });
        }

        let data_path = format!("/sdcard/Android/data/{}", package_name);
        let backup_dir = std::env::temp_dir().join("veteran_backup").join(package_name);
        let backup_existed = backup_dir.exists();

        if let Some(sender) = status_sender.as_ref() {
            let _ = sender.send("Backing up save data...".to_string());
        }

        let _ = tokio::fs::create_dir_all(&backup_dir).await;
        let pull_result = self.adb.pull_file(&data_path, &backup_dir.to_string_lossy(), serial).await;
        let has_backup = pull_result.map(|r| r.success()).unwrap_or(false);

        if let Some(sender) = status_sender.as_ref() {
            let _ = sender.send("Uninstalling old version...".to_string());
        }

        let uninstall_result = self.adb.shell(&format!("pm uninstall {}", package_name), serial).await;
        if let Err(e) = uninstall_result {
            return Ok(InstallResult {
                success: false,
                message: format!("Failed to uninstall old version: {}", e),
            });
        }

        if let Some(sender) = status_sender.as_ref() {
            let _ = sender.send("Installing new version...".to_string());
        }

        let install_result = self.adb.install_apk(&apk_path.to_string_lossy(), serial).await?;

        if !install_result.success() && !install_result.output().contains("Success") {
            return Ok(InstallResult {
                success: false,
                message: format!("Reinstall failed: {}", install_result.stderr),
            });
        }

        if has_backup {
            if let Some(sender) = status_sender.as_ref() {
                let _ = sender.send("Restoring save data...".to_string());
            }

            let push_result = self.adb.push_file(&backup_dir.to_string_lossy(), "/sdcard/Android/data/", serial).await;

            if backup_existed {
                let _ = tokio::fs::remove_dir_all(&backup_dir).await;
            }

            if let Err(e) = push_result {
                return Ok(InstallResult {
                    success: true,
                    message: format!("Reinstall succeeded but data restore failed: {}", e),
                });
            }
        }

        Ok(InstallResult {
            success: true,
            message: "Reinstall with backup: Success".to_string(),
        })
    }
}

/// Find install.txt (case-insensitive) in the game directory.
fn find_install_txt(game_dir: &Path) -> Option<PathBuf> {
    let lower = game_dir.join("install.txt");
    if lower.exists() { return Some(lower); }
    let upper = game_dir.join("Install.txt");
    if upper.exists() { return Some(upper); }
    None
}

/// Find the first .apk file in a directory (non-recursive, matching Windows behavior).
fn find_first_apk(dir: &Path) -> Option<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("apk"))
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    entries.sort();
    entries.into_iter().next()
}

/// Find 7z archives (non-recursive) in a directory.
/// Only finds .7z.001 (split) and .7z (single) archives.
fn find_archives(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut archives = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_dir() { continue; }

            if let Some(filename) = child.file_name().and_then(|n| n.to_str()) {
                let lower = filename.to_lowercase();
                if lower.ends_with(".7z.001") || lower.ends_with(".7z") {
                    archives.push(child);
                }
            }
        }
    }

    archives.sort();
    Ok(archives)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_install_txt_line_handles_valid_and_invalid_rows() {
        assert_eq!(
            InstallService::parse_install_txt_line("adb shell pm list packages").unwrap(),
            vec!["shell", "pm", "list", "packages"]
        );
        assert!(InstallService::parse_install_txt_line("echo hello").is_none());
        assert!(InstallService::parse_install_txt_line("   ").is_none());
    }

    #[test]
    fn find_first_apk_finds_apk_in_directory() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        std::fs::write(root.join("com.example.game.apk"), "apk").unwrap();
        std::fs::write(root.join("readme.txt"), "text").unwrap();

        let apk = find_first_apk(root);
        assert!(apk.is_some());
        assert!(apk.unwrap().ends_with("com.example.game.apk"));
    }

    #[test]
    fn find_first_apk_returns_none_when_no_apk() {
        let temp = tempdir().unwrap();
        std::fs::write(temp.path().join("readme.txt"), "text").unwrap();
        assert!(find_first_apk(temp.path()).is_none());
    }

    #[test]
    fn find_install_txt_case_insensitive() {
        let temp = tempdir().unwrap();

        // Neither exists
        assert!(find_install_txt(temp.path()).is_none());

        // Lowercase
        std::fs::write(temp.path().join("install.txt"), "adb shell echo hi").unwrap();
        assert!(find_install_txt(temp.path()).is_some());
    }

    #[test]
    fn find_archives_finds_7z_files() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        std::fs::write(root.join("abc123.7z.001"), "data").unwrap();
        std::fs::write(root.join("abc123.7z.002"), "data").unwrap();
        std::fs::write(root.join("single.7z"), "data").unwrap();
        std::fs::write(root.join("readme.txt"), "text").unwrap();

        let archives = find_archives(root).unwrap();
        // Should find .7z.001 and .7z, but NOT .7z.002
        assert_eq!(archives.len(), 2);
    }

}
