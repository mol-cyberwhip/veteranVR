//! Typed API Response Types
//!
//! This module provides strongly-typed structs for all IPC command responses.
//! Using these types instead of loose `serde_json::Value` ensures:
//! - Compile-time field name checking
//! - Automatic TypeScript type generation via specta
//! - No runtime mismatches between Rust and frontend

use serde::{Deserialize, Serialize};

// ============================================================================
// Common Response Types
// ============================================================================

/// Generic success/error response for simple operations
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OperationResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Generic boolean result with optional data
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BoolResult {
    pub value: bool,
}

/// Response with a single string value
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StringResult {
    pub value: String,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// ============================================================================
// Backend State Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackendReadyState {
    pub ready: bool,
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackendRecoverResult {
    pub recovered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ============================================================================
// Settings Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SettingsResponse {
    pub download_dir: String,
    pub auto_install: bool,
    pub auto_backup: bool,
    pub backup_dir: String,
    pub theme: String,
    pub language: String,
    pub enable_notifications: bool,
    pub concurrent_downloads: u32,
    pub favorited_games: Vec<String>,
    pub wireless_auto_reconnect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SettingsUpdateResponse {
    pub updated: bool,
    pub settings: SettingsResponse,
}

// ============================================================================
// Catalog Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogStatus {
    pub synced: bool,
    pub source: String,
    pub game_count: u32,
    pub has_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_base_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_error: Option<String>,
    pub thumbnails_dir: String,
    pub notes_dir: String,
    pub cache_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_age_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_stale: Option<bool>,
    pub sync_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogLoadCacheResult {
    pub loaded: bool,
    pub status: CatalogStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogSyncResult {
    pub synced: bool,
    pub status: CatalogStatus,
}

/// A game in catalog search results (condensed view)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogSearchGame {
    pub package_name: String,
    pub release_name: String,
    pub game_name: String,
    pub version_code: String,
    pub version_name: String,
    pub size: String,
    pub last_updated: String,
    pub downloads: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogSearchResult {
    pub games: Vec<CatalogSearchGame>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub query: String,
}

/// Full game details (used when fetching single game)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogGameDetail {
    pub game_name: String,
    pub release_name: String,
    pub package_name: String,
    pub version_code: String,
    pub version_name: String,
    pub size: String,
    pub last_updated: String,
    pub downloads: String,
    pub release_apk_path: String,
    pub thumbnail_path: String,
    pub thumbnail_exists: bool,
    pub note_path: String,
    pub note_excerpt: String,
    pub note_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogGameVersions {
    pub games: Vec<CatalogSearchGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogThumbnailPath {
    pub thumbnail_path: String,
    pub thumbnail_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CatalogNote {
    pub note: String,
}

/// A game in the library view (with UI-specific fields)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LibraryGame {
    pub package_name: String,
    pub release_name: String,
    pub game_name: String,
    pub size: String,
    pub last_updated: String,
    pub version_code: String,
    pub downloads: String,
    pub is_favorite: bool,
    pub is_new: bool,
    pub popularity_rank: i32,
    pub is_downloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LibraryResult {
    pub games: Vec<LibraryGame>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub query: String,
    pub sort_by: String,
    pub sort_ascending: bool,
    pub filter: String,
    pub favorites_count: u32,
}

// ============================================================================
// Favorites Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FavoritesToggleResult {
    pub package_name: String,
    pub is_favorite: bool,
    pub favorites: Vec<String>,
}

// ============================================================================
// Download Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadQueueItem {
    pub package_name: String,
    pub release_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_name: Option<String>,
    pub status: String,
    pub progress_percent: f64,
    pub speed: String,
    pub eta: String,
    pub bytes_transferred: f64,
    pub total_bytes: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadQueueStatus {
    pub queue: Vec<DownloadQueueItem>,
    pub queued_count: u32,
    pub total_count: u32,
    pub processing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_download: Option<DownloadQueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadQueueAddResult {
    pub added: bool,
    pub retried: bool,
    pub package_name: String,
    pub queue_length: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadQueueRemoveResult {
    pub removed: bool,
    pub package_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadStartResult {
    pub started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadCancelResult {
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadRetryResult {
    pub retried: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadLocation {
    pub download_dir: String,
    pub path: String,
    pub exists: bool,
    pub free_bytes: f64,
    pub file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LocalDownloadFile {
    pub name: String,
    pub package_name: String,
    pub size_bytes: f64,
    pub modified_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadListLocalResult {
    pub files: Vec<LocalDownloadFile>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadCheckLocalResult {
    pub package_name: String,
    pub has_local_files: bool,
    pub local_size_bytes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadOpenFolderResult {
    pub opened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadDeleteResult {
    pub deleted: bool,
    pub freed_bytes: f64,
}

// ============================================================================
// Install Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstallGameResult {
    pub operation_id: String,
    pub package_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstallStatus {
    pub is_installing: bool,
    pub current_package: Option<String>,
    pub progress_percent: f64,
    pub status_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstallCancelResult {
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstallLocalResult {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Device/App Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstalledApp {
    pub package_name: String,
    pub app_name: String,
    pub version_code: String,
    pub version_name: String,
    pub is_system_app: bool,
    pub install_time: Option<f64>,
    pub last_update_time: Option<f64>,
    pub in_catalog: bool,
    pub game_name: Option<String>,
    pub catalog_version_code: Option<String>,
    pub installed_version_code: Option<String>,
    pub size: Option<String>,
    pub update_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InstalledAppsResult {
    pub apps: Vec<InstalledApp>,
    pub count: u32,
    pub has_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AppVersionResult {
    pub version_code: String,
    pub version_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UninstallResult {
    pub uninstalled: bool,
    pub package_name: String,
    pub message: Option<String>,
}

// ============================================================================
// Device State Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    pub model: String,
    pub product: String,
    pub is_selected: bool,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceStorage {
    pub total_mb: f64,
    pub used_mb: f64,
    pub free_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceBattery {
    pub level_percent: Option<u32>,
    pub status: String,
    pub is_charging: bool,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceWirelessState {
    pub saved_endpoint: Option<String>,
    pub auto_reconnect_enabled: bool,
    pub last_endpoint: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_attempt_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceKeepAwake {
    pub enabled: bool,
    pub interval_seconds: u32,
    pub active_count: u32,
    pub active_operation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceState {
    pub status: String,
    pub status_message: String,
    pub troubleshooting: String,
    pub can_download: bool,
    pub can_install: bool,
    pub download_only_mode: bool,
    pub selected_serial: Option<String>,
    pub selection_source: String,
    pub devices: Vec<DeviceInfo>,
    pub storage: Option<DeviceStorage>,
    pub battery: Option<DeviceBattery>,
    pub wireless: DeviceWirelessState,
    pub keep_awake: DeviceKeepAwake,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceSelectResult {
    pub selected: bool,
    pub serial: String,
    pub device_state: DeviceState,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeviceClearSelectionResult {
    pub cleared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WirelessConnectResult {
    pub connected: bool,
    pub endpoint: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WirelessDisconnectResult {
    pub disconnected: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WirelessReconnectResult {
    pub reconnected: bool,
    pub endpoint: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WirelessEnableTcpipResult {
    pub success: bool,
    pub ip_address: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WirelessScanResult {
    pub devices: Vec<String>,
    pub message: String,
}

// ============================================================================
// Log Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogEntry {
    pub timestamp: f64,
    pub level: String,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogEntriesResult {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogExportResult {
    pub exported: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogResetResult {
    pub reset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogUploadPayload {
    pub payload: String,
}

// ============================================================================
// Window State Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WindowStateResult {
    pub state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WindowStateSetResult {
    pub saved: bool,
}

// ============================================================================
// Operation Tracking Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OperationStateHistoryEntry {
    pub version: u32,
    pub state: String,
    pub entered_at: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OperationProgress {
    pub percent: f64,
    pub completed_steps: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OperationKeepAwake {
    pub enabled: bool,
    pub interval_seconds: u32,
    pub ticks_sent: u32,
    pub last_sent_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OperationStatus {
    pub operation_id: String,
    pub operation: String,
    pub state: String,
    pub state_version: u32,
    pub state_history: Vec<OperationStateHistoryEntry>,
    pub progress: OperationProgress,
    pub cancel_requested: bool,
    pub cancel_requested_at: Option<f64>,
    pub terminal: bool,
    pub terminal_at: Option<f64>,
    pub keep_awake: Option<OperationKeepAwake>,
}

// ============================================================================
// Backup Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackupInfo {
    pub package_name: String,
    pub backup_path: String,
    pub size_bytes: f64,
    pub created_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ListBackupsResult {
    pub backups: Vec<BackupInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackupAppResult {
    pub operation_id: String,
    pub package_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RestoreAppResult {
    pub operation_id: String,
    pub package_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeleteBackupResult {
    pub deleted: bool,
    pub backup_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BulkBackupResult {
    pub operation_id: String,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BulkRestoreResult {
    pub operation_id: String,
    pub backup_count: u32,
}

// ============================================================================
// Update Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateInfo {
    pub package_name: String,
    pub current_version: String,
    pub available_version: String,
    pub update_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DetectUpdatesResult {
    pub has_updates: bool,
    pub updates: Vec<UpdateInfo>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateAppResult {
    pub operation_id: String,
    pub package_name: String,
}

// ============================================================================
// Media & Performance Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MediaFile {
    pub name: String,
    pub path: String,
    pub size_bytes: f64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MediaListResult {
    pub files: Vec<MediaFile>,
    pub total_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MediaTransferResult {
    pub transferred: bool,
    pub bytes_transferred: f64,
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PerformanceProfile {
    pub profile_name: String,
    pub cpu_limit_percent: u32,
    pub memory_limit_mb: u32,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PerformanceProfileResult {
    pub profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PerformanceProfileSetResult {
    pub saved: bool,
}

// ============================================================================
// ADB Console Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AdbConsoleExecuteResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AdbConsoleHistoryResult {
    pub commands: Vec<String>,
}

// ============================================================================
// Discovery & Metadata Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewAppInfo {
    pub package_name: String,
    pub app_name: String,
    pub install_date: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewAppsDiscoveryResult {
    pub new_apps: Vec<NewAppInfo>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DonationMetadata {
    pub donation_url: String,
    pub qr_code_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DonationMetadataResult {
    pub metadata: DonationMetadata,
}

// ============================================================================
// Privacy Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PrivacyStatus {
    pub uuid: String,
    pub temp_dir_size_bytes: f64,
    pub can_cleanup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PrivacyUuidRotateResult {
    pub rotated: bool,
    pub new_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PrivacyTempCleanupResult {
    pub cleaned: bool,
    pub freed_bytes: f64,
}

// ============================================================================
// Offline Mode Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OfflineModeStatus {
    pub enabled: bool,
    pub last_sync_at: Option<f64>,
    pub cache_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OfflineModeSetResult {
    pub enabled: bool,
}

// ============================================================================
// Crash Reporting Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CrashInfo {
    pub has_crash: bool,
    pub crash_id: Option<String>,
    pub timestamp: Option<f64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CrashReportResult {
    pub reported: bool,
    pub report_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CrashDismissResult {
    pub dismissed: bool,
}
