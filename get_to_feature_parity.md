# Feature Parity Plan: Windows Sideloader -> Rookie Desktop

This document outlines the roadmap to achieve feature parity between the existing Windows Forms application and the new Rust/Tauri desktop port. The focus is on implementing missing backend logic (stubs), enhancing the ADB/Device interaction layer, and bringing essential UI interactions (Drag & Drop, Media) to the frontend using vanilla JavaScript.

## Test-Driven Development (TDD) Approach

We will strictly follow a Test-Driven Development workflow. The Rust/Tauri stack, combined with `specta` for type generation, allows us to define precise contracts and behavior before implementation.

**Workflow for each task:**
1.  **Define:** Identify the missing feature (e.g., "Backup App").
2.  **Test:** Write a failing test case in `src-tauri/src/ipc/tests.rs` or a new service-specific test file. This test should:
    *   Instantiate the service or mock the command.
    *   Execute the function.
    *   Assert the expected outcome (e.g., "file created at X", "event emitted", "error returned").
3.  **Implement:** Write the minimal code required to pass the test.
4.  **Refactor:** Clean up the code while ensuring tests still pass.

## Architecture Philosophy for Parity
To match the Windows version's robustness while maintaining Rust's safety:
1.  **Service-Oriented Backend:** Logic currently stubbed in `commands.rs` will be moved into dedicated services in `src/services/` (e.g., `BackupService`, `LogService`, `UpdateService`).
2.  **Async/Event-Driven:** Long-running operations (Bulk Backup, Restore) will utilize the existing `push_operation_event` pattern to report progress to the frontend, ensuring the UI never freezes.
3.  **Vanilla Frontend (For now):** We will implement complex UI interactions (Drag & Drop) using standard HTML5 APIs to avoid the complexity of introducing a framework like React/Vue at this stage.

## Implementation Decisions

**File Structure:**
- New services: `desktop/src-tauri/src/services/`
- Service tests: Co-located with services (e.g., `services/log.rs` and `services/log_test.rs`)
- Integration tests: `desktop/src-tauri/tests/` for end-to-end IPC tests

**Log Format:**
- Structured JSON with fields: `timestamp`, `level`, `target`, `message`
- Output to `~/.rookie/rookie.log` with rotation (max 5 files, 10MB each)

**Crash Detection:**
- Lockfile: `~/.rookie/.lock`
- On startup: Check if lock exists -> show crash dialog
- On clean exit: Remove lockfile

**Event Naming:**
- Format: `domain.action` (e.g., `backup.started`, `backup.progress`, `backup.completed`)

**Testing Strategy:**
- Unit tests: Mock ADB/Rclone at trait level using `mockall` or manual mocks
- Integration tests: Use temp directories and process mocks
- Run from: `desktop/src-tauri/` via `cargo test`

**Version Comparison:**
- Version codes are integers (as defined in `Game` model)
- Simple numeric comparison: remote > local = update available

**Bulk Operations:**
- Fail fast: Stop on first error, report which package failed

---

## Milestone 1: Diagnostics & Reliability
**Goal:** Ensure we can debug issues and detect crashes, matching the Windows version's `CrashLogPath` and logging capabilities.

1.  **Structured Logging System:**
    *   Replace `println!` with `log` or `tracing`.
    *   Implement a `LogService` that writes to a rolling file (e.g., `rookie.log`).
    *   Capture standard output/error from Rclone and ADB subprocesses into this log.
2.  **Crash Detection:**
    *   On startup, check for a "lock file" or "dirty exit" flag from the previous run.
    *   If found, signal the frontend to show the "Crash Detected" dialog (already in HTML).
    *   Implement `backend_crash_report` to bundle the log file and upload it (or prepare it).

## Milestone 2: Data Preservation (Backup & Restore)
**Goal:** Implement the "Backup", "Restore", and "Bulk" operations which are currently stubs.

1.  **ADB File Operations:**
    *   Extend `AdbService` with `pull_file_with_progress` and `push_file_with_progress`.
    *   These must parse ADB's output or monitor file size growth to report progress percentages.
2.  **Backup Service:**
    *   Create `BackupService`.
    *   Implement `backup_game(package, include_obb)`:
        *   Resolve package path (`/sdcard/Android/data/...`).
        *   Resolve OBB path.
        *   Pull to `backup_dir/timestamp/package/`.
        *   Emit `backup.progress` events.
    *   Implement `restore_game(package, source_path)`:
        *   Push files back to device.
        *   Emit `restore.progress` events.
3.  **Bulk Orchestration:**
    *   Implement the loop for `backend_bulk_backup`:
        *   Iterate through selected packages.
        *   Run `backup_game` sequentially.
        *   Handle individual failures without stopping the whole batch.

## Milestone 3: Update Management
**Goal:** Enable the "Updates Available" view.

1.  **Version Comparison Logic:**
    *   Create `UpdateService`.
    *   Implement `detect_updates(installed_apps, catalog)`:
        *   Iterate `installed_apps`.
        *   Find matching `catalog` entry.
        *   Compare `version_code`.
        *   Return list of `UpdateItem`.
2.  **Frontend Integration:**
    *   Wire `backend_detect_updates` to the "Check Updates" button.
    *   Populate the `#updates-list` container in the UI.

## Milestone 4: Frontend Polish (Parity Features)
**Goal:** Add specific UI interactions that power users rely on.

1.  **Drag & Drop Queue Management:**
    *   The `ModernQueuePanel` in Windows allows reordering.
    *   Use HTML5 Drag and Drop API on the `#download-queue-list` items.
    *   On `drop`, calculate new index and call `backend_download_queue_reorder`.
2.  **Game Media (Trailers):**
    *   Update `CatalogGameDetail` struct to include `video_url` (parse from public config/metadata).
    *   In `index.html`, add a `<video>` or `<iframe>` container to the game details pane.
    *   Load the trailer when a game is selected.

---

## Verifiable Task List (TDD)

```bash
# Verify from desktop/src-tauri/ with: cargo test && npm run build
```

### Milestone 1: Diagnostics & Reliability

**Logging**
- [ ] **Test:** Write unit tests for `LogService::new()` verifying log directory creation at `~/.rookie/`
- [ ] **Test:** Write tests for `LogService::write()` verifying JSON format with required fields (timestamp, level, message)
- [ ] **Test:** Write tests for rotation logic (files > 10MB trigger rotation, max 5 files)
- [ ] **Impl:** Create `src/services/log.rs` with `LogService` struct
- [ ] **Impl:** Implement JSON line format: `{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"..."}`
- [ ] **Impl:** Replace all `eprintln!` and `println!` calls in commands.rs with `LogService::info/error`

**Crash Detection**
- [ ] **Test:** Write tests for `CrashService::check_previous_session()` detecting existing lockfile,  **Test:** Write tests for `CrashService::mark_session_start()` creating `~/.rookie/.lock`,  **Test:** Write tests for `CrashService::mark_session_end()` removing lockfile,  **Impl:** Create `src/services/crash.rs` with `CrashService`,  **Impl:** Integrate into `AppState::new()` to check on startup,  **Impl:** Wire `backend_crash_check` to read `CrashService` state

### Milestone 2: Data Preservation

**ADB Progress Operations**
- [ ] **Test:** Write tests for `AdbService::pull_with_progress()` parsing ADB transfer output
- [ ] **Test:** Write tests for progress callback emission (percentage updates)
- [ ] **Impl:** Add `pull_with_progress(local, remote, progress_fn)` to `AdbService`
- [ ] **Impl:** Add `push_with_progress(local, remote, progress_fn)` to `AdbService`

**Backup Service**
- [ ] **Test:** Write tests for `BackupService::backup_app()` resolving correct paths
- [ ] **Test:** Write tests for `backup.started`, `backup.progress`, `backup.completed` event emission
- [ ] **Test:** Write tests for OBB inclusion logic (when include_obb=true)
- [ ] **Test:** Write tests for backup directory structure: `backup_dir/timestamp/package_name/`
- [ ] **Impl:** Create `src/services/backup.rs` with `BackupService`
- [ ] **Impl:** Implement `backup_game(package, include_obb) -> Result<BackupResult>`
- [ ] **Impl:** Wire `backend_backup_app` command to call `BackupService`

**Restore Service**
- [ ] **Test:** Write tests for `BackupService::restore_app()` validating backup exists
- [ ] **Test:** Write tests for `restore.started`, `restore.progress`, `restore.completed` events
- [ ] **Test:** Write tests for OBB restoration
- [ ] **Impl:** Implement `restore_game(package, backup_path) -> Result<RestoreResult>`
- [ ] **Impl:** Wire `backend_restore_app` command

**Bulk Operations**
- [ ] **Test:** Write tests for `bulk_backup()` iterating packages and emitting sequence events
- [ ] **Test:** Write tests for fail-fast behavior (first error stops batch)
- [ ] **Test:** Write tests for partial results (which succeeded before failure)
- [ ] **Impl:** Implement `BackupService::bulk_backup(packages, include_obb) -> Result<BulkResult>`
- [ ] **Impl:** Implement `BackupService::bulk_restore(backups) -> Result<BulkResult>`
- [ ] **Impl:** Wire `backend_bulk_backup` and `backend_bulk_restore` commands

### Milestone 3: Update Management

**Update Detection**
- [ ] **Test:** Write tests for `UpdateService::detect_updates()` with mock catalog and installed apps
- [ ] **Test:** Write tests for version comparison logic (remote 100 > local 50 = update)
- [ ] **Test:** Write tests for apps not in catalog (no update check)
- [ ] **Test:** Write tests for apps already up to date (remote == local)
- [ ] **Impl:** Create `src/services/update.rs` with `UpdateService`
- [ ] **Impl:** Implement `detect_updates(installed_apps, catalog) -> Vec<UpdateItem>`
- [ ] **Impl:** Wire `backend_detect_updates` to return real data instead of empty list

### Milestone 4: Frontend Polish

**Drag & Drop**
- [ ] **Impl:** Add `draggable="true"` and event handlers to queue items in `index.html`
- [ ] **Impl:** Implement `dragstart`, `dragover`, `drop` handlers
- [ ] **Impl:** Calculate new index on drop and call `backend_download_queue_reorder`
- [ ] **Test:** Manual verification: drag item to new position, verify order persists after refresh

**Game Media**
- [ ] **Impl:** Add `video_url: Option<String>` to `CatalogGameDetail` model
- [ ] **Impl:** Parse video URL from catalog metadata
- [ ] **Impl:** Add video player container to game details view in `index.html`
- [ ] **Impl:** Load trailer when selecting a game with video_url

### Final Integration

- [ ] **Golden Path Test:** Manual end-to-end test:
  1. Download a game
  2. Install to device
  3. Backup save data
  4. Uninstall game
  5. Reinstall game
  6. Restore save data
  7. Verify data integrity
- [ ] **Regression Test:** Verify `cargo test` passes with all new tests
- [ ] **Build Verification:** `npm run build` produces working app
