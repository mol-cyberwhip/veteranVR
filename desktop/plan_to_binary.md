# Plan: Bundle adb, rclone, and 7z into a Single Tauri App Binary

## Context

The app currently shells out to `adb`, `rclone`, and `7z` via `Command::new("adb")` etc., expecting them to be on the user's PATH. This means users must install these tools separately. We want to bundle them into the app so it ships self-contained.

## Approach: Tauri `externalBin` Sidecars

Tauri 2 has built-in support for bundling external binaries via `bundle.externalBin` in `tauri.conf.json`. Binaries are placed in `src-tauri/binaries/` with a target-triple suffix (e.g., `adb-aarch64-apple-darwin`), and Tauri automatically selects and bundles the correct one per platform. At runtime, they end up in the app's resource directory.

## Steps

### 1. Create binary directory and download script

Create `src-tauri/binaries/` directory and a `scripts/download-binaries.sh` script that:
- Takes a target triple as argument (e.g., `aarch64-apple-darwin`)
- Downloads platform-appropriate binaries:
  - **adb**: From Android SDK Platform Tools
  - **rclone**: From rclone.org downloads (use standalone `rclone` binary)
  - **7z**: Use `7zz` (standalone, no DLL dependency) from 7-zip.org
- Renames each to `<name>-<target-triple>[.exe]` in `src-tauri/binaries/`

Naming convention required by Tauri:
```
src-tauri/binaries/
  adb-x86_64-apple-darwin
  adb-aarch64-apple-darwin
  adb-x86_64-pc-windows-msvc.exe
  adb-x86_64-unknown-linux-gnu
  rclone-x86_64-apple-darwin
  ... (same pattern for rclone and 7z)
```

### 2. Update `tauri.conf.json`

- Set `bundle.active` to `true`
- Add `externalBin` array

```json
"bundle": {
  "active": true,
  "externalBin": [
    "binaries/adb",
    "binaries/rclone",
    "binaries/7z"
  ],
  "icon": ["icons/icon.icns", "icons/icon.png"]
}
```

### 3. Create `src-tauri/src/services/binary_paths.rs` (new file)

Centralized module to resolve sidecar binary paths at runtime:
- Store the resource dir path in a `OnceLock<PathBuf>` on app startup
- Provide `init(app_handle)` to set the path from Tauri's `app.path().resource_dir()`
- Provide `adb()`, `rclone()`, `sevenz()` helpers that return full `PathBuf` to the binary
- Fall back to bare command name (PATH lookup) when resource dir isn't set (dev mode)

Add `pub mod binary_paths;` to `src-tauri/src/services/mod.rs`.

### 4. Initialize binary paths at app startup

**File**: `src-tauri/src/lib.rs`

Move `AppState::new()` into `.setup()` so it runs after `binary_paths::init()`:

```rust
pub fn run() {
    let app = register_invoke_handler(tauri::Builder::default())
        .setup(|app| {
            crate::services::binary_paths::init(app.handle());
            app.manage(AppState::new());

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let _ = crate::ipc::commands::backend_catalog_sync(state, Some(false)).await;
            });
            Ok(())
        })
        // ... rest unchanged
```

### 5. Update CLI tool call sites

**`src-tauri/src/services/adb.rs`** — Replace 5 instances of `Command::new("adb")`:
- Line 76 (`start_server`)
- Line 165 (`disconnect_wireless`)
- Lines 216 (`install_apk` push)
- Line 292 (`push_dir`)
- Line 560 (recovery `start-server`)

Each becomes: `Command::new(crate::services::binary_paths::adb())`

**`src-tauri/src/services/extract.rs`** — Line 31:
`Command::new("7z")` → `Command::new(crate::services::binary_paths::sevenz())`

**`src-tauri/src/ipc/commands.rs`** — Line 87:
`RcloneService::new(None)` → `RcloneService::new(Some(binary_paths::rclone().to_string_lossy().to_string()))`
(Same change in `new_async()` at line 46)

### 6. Update `build.rs` (minor)

Add `cargo:rerun-if-changed=binaries` so builds re-trigger when binaries change.

## Files Modified

| File | Change |
|------|--------|
| `src-tauri/tauri.conf.json` | Add `externalBin`, set `bundle.active: true` |
| `src-tauri/src/services/mod.rs` | Add `pub mod binary_paths;` |
| `src-tauri/src/services/binary_paths.rs` | **New** — sidecar path resolution |
| `src-tauri/src/lib.rs` | Move state creation into `.setup()`, call `binary_paths::init()` |
| `src-tauri/src/services/adb.rs` | 5x `Command::new("adb")` → resolved path |
| `src-tauri/src/services/extract.rs` | 1x `Command::new("7z")` → resolved path |
| `src-tauri/src/ipc/commands.rs` | Pass resolved rclone path in `new()` and `new_async()` |
| `src-tauri/build.rs` | Add rerun-if-changed for binaries dir |
| `scripts/download-binaries.sh` | **New** — binary download/rename script |

## Notable Considerations

- **Binary size**: adb ~6MB, rclone ~50-60MB, 7z ~2MB. Total ~70MB added per platform.
- **Dev mode**: Falls back to PATH lookup, so no change to developer workflow.
- **macOS code signing**: Tauri auto-signs sidecar binaries declared in `externalBin`.
- **Windows 7z**: Use `7zz.exe` (standalone) instead of `7z.exe` (needs `7z.dll`).
- **ADB server conflict**: If user has their own adb running on port 5037, there could be conflicts. This is a pre-existing issue, not introduced by bundling.

## Verification

1. Run `scripts/download-binaries.sh` for current platform
2. `cargo tauri build` — verify binaries appear in the app bundle
3. Run the built app — verify adb, rclone, and 7z operations work (connect device, download a game, extract archive)
4. `cargo tauri dev` — verify dev mode still works via PATH fallback
