use std::path::PathBuf;
use std::sync::OnceLock;

static BINARY_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the binary paths module with the app's binary directory.
/// Must be called during app setup before any binary resolution.
pub fn init(app_handle: &tauri::AppHandle) {
    use tauri::Manager;
    // Tauri sidecar binaries are extracted to the same directory as the main executable
    // On macOS: Contents/MacOS/
    // On Linux/Windows: same directory as the app binary
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(bin_dir) = exe_path.parent() {
            let _ = BINARY_DIR.set(bin_dir.to_path_buf());
        }
    }
}

fn resolve(name: &str) -> PathBuf {
    // First, check if the binary exists in the bundled location
    if let Some(bin_dir) = BINARY_DIR.get() {
        let candidate = bin_dir.join(name);
        let is_real = candidate
            .metadata()
            .map(|m| m.is_file() && m.len() > 0)
            .unwrap_or(false);
        if is_real {
            return candidate;
        }
    }
    // Dev mode fallback: use PATH lookup
    PathBuf::from(name)
}

pub fn adb() -> PathBuf {
    resolve("adb")
}

pub fn rclone() -> PathBuf {
    resolve("rclone")
}

pub fn sevenz() -> PathBuf {
    resolve("7z")
}
