use std::path::PathBuf;
use std::sync::OnceLock;

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the binary paths module with the app's resource directory.
/// Must be called during app setup before any binary resolution.
pub fn init(app_handle: &tauri::AppHandle) {
    use tauri::Manager;
    if let Ok(dir) = app_handle.path().resource_dir() {
        let _ = RESOURCE_DIR.set(dir);
    }
}

fn resolve(name: &str) -> PathBuf {
    if let Some(dir) = RESOURCE_DIR.get() {
        let candidate = dir.join(name);
        // Only use the bundled binary if it exists and is a real binary (not a 0-byte stub).
        // During dev builds, empty placeholder files are used to satisfy Tauri's build check,
        // so we fall through to the system PATH in that case.
        let is_real = candidate
            .metadata()
            .map(|m| m.len() > 0)
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
