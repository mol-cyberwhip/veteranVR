# Comparison: Windows Sideloader vs. Rookie Desktop (Tauri)

The analysis of `@windows-sideloader` (the reference implementation) and `@desktop` (the target Tauri port) reveals significant disparities. The `@desktop` project appears to be a functional "skeleton" or "bootstrap" implementation: it handles the core loop (downloading and installing games via Rclone/ADB) but lacks most of the advanced features and polish present in the Windows version. Many backend commands in the desktop app are currently stubs (returning fake success responses).

### 1. Concerning Mismatches (Features Missing or Stubbed in Desktop)

#### **A. Critical Logic Missing in Backend**
The following features have UI controls in the desktop app, but their backend implementations are **stubs** (they do nothing but return a success ID).

| Feature | Windows Implementation (`Sideloader.cs` / `MainForm.cs`) | Desktop Implementation (`commands.rs`) |
| :--- | :--- | :--- |
| **Backup App** | Fully implemented. Backs up save data and OBBs to local storage using `adb pull`. | **STUB**: `backend_backup_app` generates an ID but performs no operations. |
| **Restore App** | Fully implemented. Pushes backed-up data to device. | **STUB**: `backend_restore_app` is a stub. |
| **Bulk Operations** | Supports bulk backup/restore of multiple apps. | **STUB**: `backend_bulk_backup`/`restore` are stubs. |
| **Update Checking** | Checks for game updates against the remote version. | **STUB**: `backend_detect_updates` always returns "no updates". |
| **Crash Reporting** | Detects crashes and allows uploading logs (`Sideloader.CrashLogPath`). | **STUB**: `backend_crash_check`/`report` are stubs. |
| **Log Management** | Real-time logging to file and UI. | **STUB**: `backend_log_export`/`reset` are stubs. |

#### **B. Frontend & UI Gaps**
The desktop frontend is currently a single HTML file (`index.html`) with inline modules, lacking a modern build process (e.g., React/Vue/Svelte).

*   **Drag-and-Drop Queue Management:**
    *   **Windows:** Features a custom `ModernQueuePanel` that supports drag-and-drop reordering of the download queue.
    *   **Desktop:** The backend (`download.rs`) supports reordering via an API (`reorder_queue`), but the **frontend has no drag-and-drop implementation** or event listeners for it.

*   **YouTube Game Previews:**
    *   **Windows:** Embeds a `WebView2` browser to play trailer videos for the selected game.
    *   **Desktop:** The frontend contains **no video player or iframe** component to show game previews.

### 2. Implementation Comparison

| Category | Windows Sideloader (Reference) | Desktop (Target) |
| :--- | :--- | :--- |
| **Architecture** | C# .NET Windows Forms | Rust (Tauri) + Vanilla JS/HTML |
| **Download Engine** | Wrapper around `rclone.exe` binary. | Native Rust wrapper (`rclone.rs`) calling binary. |
| **ADB Engine** | Wrapper around `adb.exe` binary. | Native Rust wrapper (`adb.rs`) calling binary. |
| **Queue Logic** | `BindingList<string>` in UI thread + `ModernQueuePanel`. | `DownloadService` (Rust) with async worker loop. |
| **Settings** | `SettingsManager` (XML/Properties). | `SettingsService` (JSON) - **Implemented**. |
| **Game Catalog** | Parses remote `vrp-public.json` / text files. | `CatalogService` (Rust) parses metadata - **Implemented**. |

### 3. Conclusion
The `@desktop` application is currently a **Minimum Viable Product (MVP)** focused on the "happy path" of installing games. It is not yet a feature-complete port. To reach parity, the following work is required:

1.  **Implement Backend Logic:** Fill in the empty stub functions in `src-tauri/src/ipc/commands.rs` for Backup, Restore, and Update checking.
2.  **Frontend Engineering:** Implement the drag-and-drop interface for the queue (likely requiring a move to a more robust frontend framework or significant vanilla JS effort).
3.  **Media Features:** Add the YouTube video player component to the game details view.
