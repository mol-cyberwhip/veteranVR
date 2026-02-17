use std::collections::HashMap;
use regex::Regex;
use crate::models::game::Game;
use crate::models::responses::*;
use crate::services::adb::AdbService;
use serde_json::Value;
use crate::services::catalog::CatalogService;
use crate::services::config::ConfigService;
use crate::services::download::{DownloadService, DownloadStatus};
use crate::services::extract::ExtractService;
use crate::services::install::InstallService;
use crate::services::rclone::RcloneService;
use crate::services::settings::SettingsService;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, Wry};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use chrono::Utc;
use specta::specta;

#[derive(Clone)]
pub struct AppState {
    settings: SettingsService,
    config: ConfigService,
    catalog: Arc<RwLock<CatalogService>>,
    adb: AdbService,
    download: Arc<Mutex<DownloadService>>,
    pub rclone: Arc<RcloneService>,
    install: InstallService,
    selected_serial: Arc<RwLock<Option<String>>>,
    pub youtube_cache: Arc<Mutex<HashMap<String, Option<String>>>>,
    events: Arc<Mutex<Vec<Value>>>,
}

impl AppState {
    pub async fn new_async() -> Self {
        let settings_service = SettingsService::new();
        let settings = settings_service.get_settings_sync();

        let cache_dir = dirs::home_dir()
            .map(|p| p.join(".veteran").join("cache"))
            .unwrap_or_else(|| PathBuf::from(".veteran").join("cache"));
        
        let config_service = ConfigService::new(Some(cache_dir.clone()));

        let download_dir = PathBuf::from(&settings.download_dir);
        let rclone = Arc::new(RcloneService::new(Some(crate::services::binary_paths::rclone().to_string_lossy().to_string())));
        
        // Load cached config immediately so downloads work before sync completes
        if let Ok(cached_config) = config_service.load_from_cache() {
            rclone.set_public_config(&cached_config);
        }
        
        let download = DownloadService::new_with_arc(rclone.clone(), download_dir, settings.bandwidth_limit_mbps);
        let adb = AdbService::new();
        let install = InstallService::new(adb.clone());
        
        // Create catalog service and load cache on startup
        let mut catalog_service = CatalogService::with_cache_dir(cache_dir);
        let _ = catalog_service.load_from_cache();

        Self {
            settings: settings_service,
            config: config_service,
            catalog: Arc::new(RwLock::new(catalog_service)),
            adb,
            download: Arc::new(Mutex::new(download)),
            rclone,
            install,
            selected_serial: Arc::new(RwLock::new(None)),
            youtube_cache: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn new() -> Self {
        // For backwards compatibility with sync code,
        // we'll need to handle this differently
        let settings_service = SettingsService::new();
        let settings = settings_service.get_settings_sync();

        let cache_dir = dirs::home_dir()
            .map(|p| p.join(".veteran").join("cache"))
            .unwrap_or_else(|| PathBuf::from(".veteran").join("cache"));

        let config_service = ConfigService::new(Some(cache_dir.clone()));

        let download_dir = PathBuf::from(&settings.download_dir);
        let rclone = Arc::new(RcloneService::new(Some(crate::services::binary_paths::rclone().to_string_lossy().to_string())));
        
        // Load cached config immediately so downloads work before sync completes
        if let Ok(cached_config) = config_service.load_from_cache() {
            rclone.set_public_config(&cached_config);
        }
        
        let download = DownloadService::new_with_arc(rclone.clone(), download_dir, settings.bandwidth_limit_mbps);
        let adb = AdbService::new();
        let install = InstallService::new(adb.clone());

        // Load catalog from cache immediately for snappy startup
        let mut catalog_service = CatalogService::with_cache_dir(cache_dir);
        let _ = catalog_service.load_from_cache();

        Self {
            settings: settings_service,
            config: config_service,
            catalog: Arc::new(RwLock::new(catalog_service)),
            adb,
            download: Arc::new(Mutex::new(download)),
            rclone,
            install,
            selected_serial: Arc::new(RwLock::new(None)),
            youtube_cache: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a minimal AppState for testing purposes
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        use crate::models::settings::Settings;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        let temp_dir = std::env::temp_dir().join(format!("veteran_test_{}_{}", std::process::id(), test_id));
        let _ = std::fs::create_dir_all(&temp_dir);
        
        let settings = Settings {
            download_dir: temp_dir.join("downloads").to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let cache_dir = temp_dir.join("cache");
        let _ = std::fs::create_dir_all(&cache_dir);
        
        // Create minimal services without file I/O
        let settings_service = SettingsService::from_settings(settings, temp_dir.join("settings.json"));
        let config_service = ConfigService::new(Some(cache_dir.clone()));
        
        let download_dir = temp_dir.join("downloads");
        let _ = std::fs::create_dir_all(&download_dir);
        
        let rclone = Arc::new(RcloneService::new(Some(crate::services::binary_paths::rclone().to_string_lossy().to_string())));
        let download = DownloadService::new_with_arc(rclone.clone(), download_dir, 0.0);
        let adb = AdbService::new();
        let install = InstallService::new(adb.clone());

        Self {
            settings: settings_service,
            config: config_service,
            catalog: Arc::new(RwLock::new(CatalogService::with_cache_dir(cache_dir))),
            adb,
            download: Arc::new(Mutex::new(download)),
            rclone,
            install,
            selected_serial: Arc::new(RwLock::new(None)),
            youtube_cache: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub async fn push_operation_event(
        &self,
        event_name: &str,
        operation_id: &str,
        operation_kind: &str,
        state: &str,
        message: &str,
        progress_percent: f64,
    ) {
        let mut events = self.events.lock().await;
        let now = Utc::now().timestamp() as f64;
        
        let terminal = state == "succeeded" || state == "failed" || state == "cancelled" || state == "completed";
        let actual_state = if state == "completed" { "succeeded" } else { state };

        events.push(json!({
            "schema_version": 1,
            "kind": "event",
            "event": event_name,
            "timestamp": now,
            "message": message,
            "operation": {
                "operation_id": operation_id,
                "operation": operation_kind,
                "state": actual_state,
                "state_version": 1,
                "state_history": [
                    {
                        "version": 1,
                        "state": actual_state,
                        "entered_at": now,
                        "reason": message
                    }
                ],
                "progress": {
                    "percent": progress_percent,
                    "completed_steps": 0,
                    "total_steps": 100
                },
                "cancel_requested": false,
                "cancel_requested_at": Value::Null,
                "terminal": terminal,
                "terminal_at": if terminal { json!(now) } else { Value::Null },
                "keep_awake": {
                    "enabled": false,
                    "interval_seconds": 30,
                    "ticks_sent": 0,
                    "last_sent_at": Value::Null
                }
            },
            "error": Value::Null,
            "extra": {}
        }));
    }

    pub async fn push_event(&self, event_name: &str, message: &str, progress_percent: f64) {
        // Legacy fallback for simple events if needed, but we should use push_operation_event
        self.push_operation_event(event_name, "00000000-0000-0000-0000-000000000000", "unknown", "running", message, progress_percent).await;
    }

    /// Test helper to access events for verification
    #[doc(hidden)]
    pub async fn get_events(&self) -> Vec<Value> {
        self.events.lock().await.clone()
    }
}

fn map_download_status(status: DownloadStatus) -> &'static str {
    match status {
        DownloadStatus::Queued => "queued",
        DownloadStatus::Downloading => "downloading",
        DownloadStatus::Paused => "paused",
        DownloadStatus::Completed => "completed",
        DownloadStatus::Failed => "failed",
        DownloadStatus::Cancelled => "cancelled",
    }
}

async fn selected_serial(state: &AppState) -> Option<String> {
    state.selected_serial.read().await.clone()
}

fn paginate(games: Vec<Game>, limit: u32, offset: u32) -> (Vec<Game>, usize) {
    let total = games.len();
    let page = games
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect::<Vec<_>>();
    (page, total)
}

/// Parse a size string like "1.5 GB" or "500 MB" to a numeric value in MB
fn parse_size_mb(size_str: &str) -> f64 {
    let size_str = size_str.trim().to_lowercase();
    
    // Handle "Unknown" or empty
    if size_str.is_empty() || size_str == "unknown" {
        return 0.0;
    }
    
    // Check for GB
    if let Some(idx) = size_str.find("gb") {
        let num_part = &size_str[..idx].trim();
        if let Ok(gb) = num_part.parse::<f64>() {
            return gb * 1024.0;
        }
    }
    
    // Check for MB
    if let Some(idx) = size_str.find("mb") {
        let num_part = &size_str[..idx].trim();
        if let Ok(mb) = num_part.parse::<f64>() {
            return mb;
        }
    }
    
    // Try parsing as raw number (assume MB)
    if let Ok(mb) = size_str.parse::<f64>() {
        return mb;
    }
    
    0.0
}

// --- Readiness & Lifecycle ---

#[tauri::command]
#[specta]
pub async fn backend_ready_state() -> Result<BackendReadyState, String> {
    Ok(BackendReadyState {
        ready: true,
        pid: Some(std::process::id()),
        version: None,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_recover() -> Result<BackendReadyState, String> {
    backend_ready_state().await
}

#[tauri::command]
#[specta]
pub async fn shutdown_backend(state: State<'_, AppState>) -> Result<(), String> {
    state.rclone.shutdown().await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- Settings ---

#[tauri::command]
#[specta]
pub async fn backend_get_settings(state: State<'_, AppState>) -> Result<SettingsResponse, String> {
    let settings = state.settings.get_settings().await;
    Ok(SettingsResponse {
        download_dir: settings.download_dir.clone(),
        auto_install: settings.delete_after_install,
        auto_backup: settings.keep_awake_during_long_ops,
        backup_dir: settings.download_dir,
        theme: "dark".to_string(),
        language: "en".to_string(),
        enable_notifications: true,
        concurrent_downloads: settings.keep_awake_interval_seconds as u32,
        favorited_games: settings.favorited_games,
        wireless_auto_reconnect: settings.wireless_adb,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_update_settings(
    state: State<'_, AppState>,
    patch: Value,
) -> Result<SettingsResponse, String> {
    state
        .settings
        .patch_settings(patch)
        .await
        .map_err(|err| err.to_string())?;
    backend_get_settings(state).await
}

// --- Catalog ---

#[tauri::command]
#[specta]
pub async fn backend_catalog_status(state: State<'_, AppState>) -> Result<CatalogStatus, String> {
    let catalog = state.catalog.read().await;
    let game_count = catalog.games().len() as u32;
    let has_games = game_count > 0;
    
    // Calculate cache age and staleness
    let cache_file = catalog.cache_dir().join("VRP-GameList.txt");
    let (cache_age_hours, cache_stale) = if cache_file.exists() {
        if let Ok(metadata) = std::fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                let age = std::time::SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default()
                    .as_secs_f64() / 3600.0;
                // Cache is stale if older than 24 hours
                let stale = age > 24.0;
                (Some(age), Some(stale))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    
    // Determine source
    let source = if has_games {
        "cache".to_string()
    } else {
        "none".to_string()
    };

    Ok(CatalogStatus {
        synced: has_games,
        source,
        game_count,
        has_config: true,
        config_base_uri: None,
        sync_error: None,
        thumbnails_dir: catalog.thumbnails_dir().display().to_string(),
        notes_dir: catalog.notes_dir().display().to_string(),
        cache_dir: catalog.cache_dir().display().to_string(),
        cache_age_hours,
        cache_stale,
        sync_in_progress: catalog.is_syncing(),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_load_cache(state: State<'_, AppState>) -> Result<CatalogLoadCacheResult, String> {
    let loaded = {
        let mut catalog = state.catalog.write().await;
        catalog.load_from_cache()
    };

    let status = backend_catalog_status(state).await?;
    Ok(CatalogLoadCacheResult { loaded, status })
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_sync(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<CatalogSyncResult, String> {
    let force = force.unwrap_or(false);
    crate::logger::log(&format!("[CATALOG] Sync triggered (force={})", force));

    // Check cache validity (4 hour rule) unless forced
    if !force {
        let cache_age = state.catalog.read().await.get_cache_age();
        if let Some(age) = cache_age {
            crate::logger::log(&format!("[CATALOG] Cache age: {:.2} hours", age));
            if age < 4.0 {
                crate::logger::log("[CATALOG] Cache is fresh (< 4h). Skipping network sync.");
                // Cache is fresh enough. Reload it to ensure we have the latest content from disk
                // (though usually it's already in memory, this handles external updates)
                let _ = backend_catalog_load_cache(state.clone()).await?;
                let status = backend_catalog_status(state).await?;
                return Ok(CatalogSyncResult {
                    synced: true,
                    status,
                });
            }
        } else {
            crate::logger::log("[CATALOG] No cache file found.");
        }
    }

    // Set sync_in_progress flag
    {
        let mut catalog = state.catalog.write().await;
        catalog.set_syncing(true);
    }
    
    crate::logger::log("[CATALOG] Starting background sync process...");
    // Perform sync in spawn_blocking to avoid blocking the async runtime
    let result = tokio::task::spawn_blocking({
        let state = state.inner().clone();
        move || {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                // 1. Fetch config
                crate::logger::log("[CATALOG] Fetching config...");
                let config = state.config.fetch_config().await.map_err(|e| e.to_string())?;
                crate::logger::log(&format!("[CATALOG] Config base_uri: {}", config.base_uri));
                crate::logger::log(&format!("[CATALOG] Config password length: {}", config.password.len()));
                
                // 2. Update rclone config
                state.rclone.set_public_config(&config);
                
                // 3. Sync metadata
                let cache_dir = state.catalog.read().await.cache_dir().to_path_buf();
                let meta_download_dir = cache_dir.join("meta_download");
                let _ = tokio::fs::create_dir_all(&meta_download_dir).await;
                
                let meta_archive = meta_download_dir.join("meta.7z");
                let extract_dir = cache_dir.join("meta_extracted");
                let mut game_list_path = extract_dir.join("VRP-GameList.txt");

                // Check modtime before sync to see if we need to extract
                let initial_modtime = if meta_archive.exists() {
                    std::fs::metadata(&meta_archive)
                        .and_then(|m| m.modified())
                        .ok()
                } else {
                    None
                };

                crate::logger::log(&format!("[CATALOG] Cache dir: {}", cache_dir.display()));
                crate::logger::log(&format!("[CATALOG] Meta download dir: {}", meta_download_dir.display()));
                crate::logger::log(&format!("[CATALOG] Meta archive path: {}", meta_archive.display()));
                crate::logger::log(&format!("[CATALOG] Meta archive exists before sync: {}", meta_archive.exists()));
                
                crate::logger::log("[CATALOG] Running rclone sync for meta.7z...");
                let result = state.rclone.sync_metadata(&meta_download_dir).await.map_err(|e| e.to_string())?;
                crate::logger::log(&format!("[CATALOG] Sync result: success={}, stdout={}", result.success(), result.stdout));
                if !result.success() {
                    crate::logger::log(&format!("[CATALOG] Metadata sync failed: {}", result.stderr));
                    return Err(format!("Metadata sync failed: {}", result.stderr));
                }
                
                // Check if file was actually downloaded
                crate::logger::log(&format!("[CATALOG] Meta archive exists after sync: {}", meta_archive.exists()));
                
                // List all files in download dir to debug
                if let Ok(entries) = std::fs::read_dir(&meta_download_dir) {
                    let files: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    crate::logger::log(&format!("[CATALOG] Files in meta_download: {:?}", files));
                }
                
                if meta_archive.exists() {
                    if let Ok(metadata) = std::fs::metadata(&meta_archive) {
                        crate::logger::log(&format!("[CATALOG] Meta archive size: {} bytes", metadata.len()));
                    }
                }
                
                // Check if we need to extract:
                // 1. If meta.7z changed (new download)
                // 2. If VRP-GameList.txt is missing (first run or deleted)
                let new_modtime = std::fs::metadata(&meta_archive)
                    .and_then(|m| m.modified())
                    .ok();
                
                let should_extract = initial_modtime != new_modtime || !game_list_path.exists();

                if should_extract {
                    crate::logger::log("[CATALOG] Extracting meta.7z...");
                    let _ = tokio::fs::create_dir_all(&extract_dir).await;
                    
                    let archive_path = meta_archive.clone();
                    let extract_path = extract_dir.clone();
                    let password = config.password.clone();
                    
                    tokio::task::spawn_blocking(move || {
                        ExtractService::extract_7z(&archive_path, &extract_path, Some(&password))
                            .map_err(|e| e.to_string())
                    }).await.map_err(|e| e.to_string())??;
                } else {
                    crate::logger::log("[CATALOG] meta.7z unchanged. Skipping extraction.");
                }

                // 4.5 Ensure thumbnails and notes are present in the main cache
                // This runs even if we skip extraction, ensuring we recover if the cache was partially cleared
                let extracted_meta = extract_dir.join(".meta");
                if extracted_meta.exists() {
                    let extracted_thumbnails = extracted_meta.join("thumbnails");
                    let extracted_notes = extracted_meta.join("notes");
                    
                    let target_thumbnails = cache_dir.join("thumbnails");
                    let target_notes = cache_dir.join("notes");
                    
                    let _ = tokio::fs::create_dir_all(&target_thumbnails).await;
                    let _ = tokio::fs::create_dir_all(&target_notes).await;
                    
                    crate::logger::log("[CATALOG] Syncing thumbnails and notes to cache...");
                    
                    // Copy thumbnails
                    if let Ok(mut entries) = tokio::fs::read_dir(&extracted_thumbnails).await {
                        let mut count = 0;
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let dest = target_thumbnails.join(entry.file_name());
                            if !dest.exists() {
                                let _ = tokio::fs::copy(entry.path(), dest).await;
                                count += 1;
                            }
                        }
                        if count > 0 {
                            crate::logger::log(&format!("[CATALOG] Copied {} new thumbnails", count));
                        }
                    }
                    
                    // Copy notes
                    if let Ok(mut entries) = tokio::fs::read_dir(&extracted_notes).await {
                        let mut count = 0;
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let dest = target_notes.join(entry.file_name());
                            if !dest.exists() {
                                let _ = tokio::fs::copy(entry.path(), dest).await;
                                count += 1;
                            }
                        }
                        if count > 0 {
                            crate::logger::log(&format!("[CATALOG] Copied {} new notes", count));
                        }
                    }
                }
                
                // 5. Load into catalog
                if !game_list_path.exists() {
                    game_list_path = extract_dir.join(".meta").join("VRP-GameList.txt");
                }
                
                if game_list_path.exists() {
                    crate::logger::log("[CATALOG] Parsing game list file...");
                    let mut catalog = state.catalog.write().await;
                    catalog.parse_game_list_file(&game_list_path).map_err(|e| e.to_string())?;
                    
                    // Cache it for offline use and update timestamp
                    let cached_path = cache_dir.join("VRP-GameList.txt");
                    let _ = tokio::fs::copy(&game_list_path, &cached_path).await;
                    
                    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&cached_path) {
                        let _ = file.set_modified(std::time::SystemTime::now());
                    }
                }
                
                crate::logger::log("[CATALOG] Sync completed successfully.");
                Ok::<(), String>(())
            })
        }
    }).await.map_err(|e| e.to_string())?;
    
    // Clear sync_in_progress flag
    {
        let mut catalog = state.catalog.write().await;
        catalog.set_syncing(false);
    }
    
    match result {
        Ok(()) => {
            let status = backend_catalog_status(state).await?;
            Ok(CatalogSyncResult { synced: true, status })
        }
        Err(e) => {
            let mut catalog = state.catalog.write().await;
            catalog.set_syncing(false);
            Err(e)
        }
    }
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_search(
    state: State<'_, AppState>,
    query: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<CatalogSearchResult, String> {
    let query = query.unwrap_or_default();
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let games = state.catalog.read().await.search(&query);
    let (games, total) = paginate(games, limit, offset);

    let games: Vec<CatalogSearchGame> = games
        .into_iter()
        .map(|game| CatalogSearchGame {
            package_name: game.package_name,
            release_name: game.release_name,
            game_name: game.game_name,
            version_code: game.version_code,
            version_name: game.version_name,
            size: game.size,
            last_updated: game.last_updated,
            downloads: game.downloads,
        })
        .collect();

    Ok(CatalogSearchResult {
        games,
        total: total as u32,
        offset,
        limit,
        query,
    })
}

#[tauri::command]
#[specta]
pub async fn search_youtube_trailer(
    state: State<'_, AppState>,
    game_name: String,
) -> Result<Option<String>, String> {
    // Check cache
    {
        let cache = state.youtube_cache.lock().await;
        if let Some(video_id) = cache.get(&game_name) {
            return Ok(video_id.clone());
        }
    }

    // Search YouTube
    let query = format!("{} VR trailer", game_name);
    let encoded_query = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    let url = format!("https://www.youtube.com/results?search_query={}", encoded_query);
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let text = response.text().await.map_err(|e| e.to_string())?;

    // Parse video ID
    let re = Regex::new(r"watch\?v=([a-zA-Z0-9_-]{11})").map_err(|e| e.to_string())?;
    let video_id = re.captures(&text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    // Update cache
    {
        let mut cache = state.youtube_cache.lock().await;
        cache.insert(game_name, video_id.clone());
    }

    Ok(video_id)
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_game_detail(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<CatalogGameDetail, String> {
    let game = state
        .catalog
        .read()
        .await
        .get_game_by_package(&package_name)
        .cloned();

    match game {
        Some(game) => {
            let catalog = state.catalog.read().await;
            let thumbnail_path = catalog.thumbnails_dir().join(format!("{}.jpg", game.package_name));
            let note_path = catalog.notes_dir().join(format!("{}.txt", game.package_name));
            let note = if note_path.exists() {
                std::fs::read_to_string(&note_path).unwrap_or_default()
            } else {
                String::new()
            };
            
            Ok(CatalogGameDetail {
                game_name: game.game_name,
                release_name: game.release_name,
                package_name: game.package_name,
                version_code: game.version_code,
                version_name: game.version_name,
                size: game.size,
                last_updated: game.last_updated,
                downloads: game.downloads,
                release_apk_path: game.release_apk_path,
                thumbnail_path: thumbnail_path.display().to_string(),
                thumbnail_exists: thumbnail_path.exists(),
                note_path: note_path.display().to_string(),
                note_excerpt: note.chars().take(200).collect(),
                note_exists: note_path.exists(),
            })
        }
        None => Err(format!("Game not found for package: {package_name}")),
    }
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_game_versions(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<CatalogGameVersions, String> {
    let games = state.catalog.read().await.get_games_by_package(&package_name);
    let games: Vec<CatalogSearchGame> = games
        .into_iter()
        .map(|game| CatalogSearchGame {
            package_name: game.package_name,
            release_name: game.release_name,
            game_name: game.game_name,
            version_code: game.version_code,
            version_name: game.version_name,
            size: game.size,
            last_updated: game.last_updated,
            downloads: game.downloads,
        })
        .collect();
    Ok(CatalogGameVersions { games })
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_thumbnail_path(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<CatalogThumbnailPath, String> {
    let catalog = state.catalog.read().await;
    let thumbnail_path = catalog.thumbnails_dir().join(format!("{package_name}.jpg"));
    Ok(CatalogThumbnailPath {
        thumbnail_path: thumbnail_path.display().to_string(),
        thumbnail_exists: thumbnail_path.exists(),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_note(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<CatalogNote, String> {
    let catalog = state.catalog.read().await;
    let path = catalog.notes_dir().join(format!("{package_name}.txt"));
    let note = if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };
    Ok(CatalogNote { note })
}

#[tauri::command]
#[specta]
pub async fn backend_catalog_library(
    state: State<'_, AppState>,
    query: Option<String>,
    sort_by: Option<String>,
    sort_ascending: Option<bool>,
    filter: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<LibraryResult, String> {
    let query = query.unwrap_or_default();
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    let sort_by = sort_by.unwrap_or_else(|| "name".to_string());
    let sort_ascending = sort_ascending.unwrap_or(true);
    let filter = filter.unwrap_or_else(|| "all".to_string());

    let favorites = state.settings.get_settings().await.favorited_games;
    let mut games = state.catalog.read().await.search(&query);

    if filter == "favorites" {
        games.retain(|g| favorites.iter().any(|pkg| pkg == &g.package_name));
    }

    match sort_by.as_str() {
        "date" => games.sort_by(|a, b| a.last_updated.cmp(&b.last_updated)),
        "size" => games.sort_by(|a, b| {
            // Parse size strings like "1.5 GB" or "500 MB" to numeric MB values
            let size_a = parse_size_mb(&a.size);
            let size_b = parse_size_mb(&b.size);
            size_a.partial_cmp(&size_b).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "popularity" => games.sort_by(|a, b| {
            // Sort by popularity_rank (lower rank = more popular)
            // Unranked games (rank 0 or negative) go to the end
            let rank_a = if a.popularity_rank > 0 { a.popularity_rank } else { i32::MAX };
            let rank_b = if b.popularity_rank > 0 { b.popularity_rank } else { i32::MAX };
            rank_a.cmp(&rank_b)
        }),
        _ => games.sort_by(|a, b| a.game_name.cmp(&b.game_name)),
    }

    if !sort_ascending {
        games.reverse();
    }

    let total = games.len();
    let mut paged = Vec::new();
    let start = offset as usize;
    let end = (start + limit as usize).min(games.len());
    
    if start < games.len() {
        let download = state.download.lock().await;
        for game in &games[start..end] {
            let is_downloaded = download.is_downloaded(game).await;
            paged.push(LibraryGame {
                package_name: game.package_name.clone(),
                release_name: game.release_name.clone(),
                game_name: game.game_name.clone(),
                size: game.size.clone(),
                last_updated: game.last_updated.clone(),
                version_code: game.version_code.clone(),
                downloads: game.downloads.clone(),
                is_favorite: favorites.iter().any(|pkg| pkg == &game.package_name),
                is_new: game.is_new,
                popularity_rank: game.popularity_rank,
                is_downloaded,
            });
        }
    }

    Ok(LibraryResult {
        games: paged,
        total: total as u32,
        offset,
        limit,
        query,
        sort_by,
        sort_ascending,
        filter,
        favorites_count: favorites.len() as u32,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_favorites_toggle(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<FavoritesToggleResult, String> {
    let mut settings = state.settings.get_settings().await;
    let exists = settings.favorited_games.iter().any(|pkg| pkg == &package_name);

    if exists {
        settings.favorited_games.retain(|pkg| pkg != &package_name);
    } else {
        settings.favorited_games.push(package_name.clone());
    }

    state
        .settings
        .update_settings(settings.clone())
        .await
        .map_err(|err| err.to_string())?;

    Ok(FavoritesToggleResult {
        package_name,
        is_favorite: !exists,
        favorites: settings.favorited_games,
    })
}

// --- Download Queue ---

#[tauri::command]
#[specta]
pub async fn backend_download_queue_status(state: State<'_, AppState>) -> Result<DownloadQueueStatus, String> {
    let download = state.download.lock().await;
    let queue_items = download.queue().await;
    let total_count = queue_items.len() as u32;
    let queued_count = queue_items.iter().filter(|i| i.status == DownloadStatus::Queued).count() as u32;

    let queue: Vec<DownloadQueueItem> = queue_items
        .iter()
        .map(|item| DownloadQueueItem {
            package_name: item.game.package_name.clone(),
            release_name: item.game.release_name.clone(),
            game_name: Some(item.game.game_name.clone()),
            status: map_download_status(item.status).to_string(),
            progress_percent: item.progress.percent,
            speed: item.progress.speed.clone(),
            eta: item.progress.eta.clone(),
            bytes_transferred: item.progress.bytes_transferred as f64,
            total_bytes: item.progress.total_bytes as f64,
            retry_count: None,
            error: if item.error.is_empty() { None } else { Some(item.error.clone()) },
        })
        .collect();

    // Find the active download (currently downloading)
    let active_download = queue_items
        .iter()
        .find(|item| item.status == DownloadStatus::Downloading)
        .map(|item| DownloadQueueItem {
            package_name: item.game.package_name.clone(),
            release_name: item.game.release_name.clone(),
            game_name: Some(item.game.game_name.clone()),
            status: map_download_status(item.status).to_string(),
            progress_percent: item.progress.percent,
            speed: item.progress.speed.clone(),
            eta: item.progress.eta.clone(),
            bytes_transferred: item.progress.bytes_transferred as f64,
            total_bytes: item.progress.total_bytes as f64,
            retry_count: None,
            error: if item.error.is_empty() { None } else { Some(item.error.clone()) },
        });

    let processing = download.is_processing().await;
    drop(download);

    Ok(DownloadQueueStatus {
        queue,
        queued_count,
        total_count,
        processing,
        active_download,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_download_queue_add(
    state: State<'_, AppState>,
    package_name: String,
    release_name: Option<String>,
) -> Result<DownloadQueueAddResult, String> {
    let maybe_game = {
        let catalog = state.catalog.read().await;
        // If release_name is provided, use it to find the exact version.
        // Otherwise, fall back to the default game for that package.
        match release_name {
            Some(ref rel) if !rel.is_empty() => catalog
                .get_game_by_package_and_release(&package_name, rel)
                .cloned(),
            _ => catalog.get_game_by_package(&package_name).cloned(),
        }
    };

    let Some(game) = maybe_game else {
        return Err(format!("No catalog game found for package: {package_name}"));
    };

    let download = state.download.lock().await;
    
    // Check if already queued - check both package and release name
    let queue_items = download.queue().await;
    let already_queued = queue_items.iter().any(|item| {
        item.game.package_name == package_name 
            && item.game.release_name == game.release_name
    });
    
    if already_queued {
        let queue_length = queue_items.len() as u32;
        return Ok(DownloadQueueAddResult {
            added: false,
            retried: false,
            package_name,
            queue_length,
            reason: Some("already_queued".to_string()),
        });
    }
    
    let added = download.add_to_queue(game).await;
    let queue_length = download.queue().await.len() as u32;
    drop(download);
    
    Ok(DownloadQueueAddResult {
        added,
        retried: false,
        package_name,
        queue_length,
        reason: None,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_download_queue_remove(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<DownloadQueueStatus, String> {
    let download = state.download.lock().await;
    let _removed = download.remove_from_queue(&package_name).await;
    drop(download);
    backend_download_queue_status(state).await
}

#[tauri::command]
#[specta]
pub async fn backend_download_queue_reorder(
    state: State<'_, AppState>,
    package_name: String,
    position: u32,
) -> Result<DownloadQueueStatus, String> {
    let download = state.download.lock().await;
    let _moved = download.reorder_queue(&package_name, position as usize).await;
    drop(download);
    backend_download_queue_status(state).await
}

#[tauri::command]
#[specta]
pub async fn backend_download_start_processing(state: State<'_, AppState>) -> Result<DownloadStartResult, String> {
    let download = state.download.lock().await;
    let download_clone = download.clone();
    let app_state = state.inner().clone();
    drop(download);

    tokio::spawn(async move {
        let _ = download_clone
            .process_queue_with_callback(move |item| {
                let app_state = app_state.clone();
                async move {
                    let event_name = match item.status {
                        DownloadStatus::Downloading => "download.progress",
                        DownloadStatus::Paused => "download.paused",
                        DownloadStatus::Completed => "download.completed",
                        DownloadStatus::Failed => "download.failed",
                        DownloadStatus::Cancelled => "download.cancelled",
                        _ => return,
                    };
                    let state = match item.status {
                        DownloadStatus::Downloading => "running",
                        DownloadStatus::Paused => "paused",
                        DownloadStatus::Completed => "succeeded",
                        DownloadStatus::Failed => "failed",
                        DownloadStatus::Cancelled => "cancelled",
                        _ => return,
                    };
                    app_state
                        .push_operation_event(
                            event_name,
                            &item.operation_id,
                            "download",
                            state,
                            &item.game.game_name,
                            item.progress.percent,
                        )
                        .await;
                }
            })
            .await;
    });
    Ok(DownloadStartResult { started: true })
}

#[tauri::command]
#[specta]
pub async fn backend_download_cancel(
    state: State<'_, AppState>,
    package_name: Option<String>,
) -> Result<DownloadCancelResult, String> {
    let download = state.download.lock().await;
    
    // If a package name is provided, remove it from the queue regardless of status.
    // If it's currently downloading, it will be cancelled as well.
    if let Some(pkg) = package_name {
        // Find if it is the current active download
        let is_active = download.queue().await.iter().any(|i| i.game.package_name == pkg && i.status == DownloadStatus::Downloading);
        
        if is_active {
            let _ = download.cancel_current().await;
        }
        
        // Remove from queue completely so it doesn't linger in UI
        let cancelled = download.remove_from_queue(&pkg).await;
        return Ok(DownloadCancelResult { cancelled });
    }

    // Fallback: cancel current active download if no package specified
    let cancelled = download
        .cancel_current()
        .await
        .map_err(|err| err.to_string())?;
    
    // Also remove the cancelled item from queue
    if cancelled {
        let queue = download.queue().await;
        if let Some(item) = queue.iter().find(|i| i.status == DownloadStatus::Cancelled) {
            download.remove_from_queue(&item.game.package_name).await;
        }
    }

    Ok(DownloadCancelResult { cancelled })
}

#[tauri::command]
#[specta]
pub async fn backend_download_retry(
    state: State<'_, AppState>,
    package_name: String,
    release_name: Option<String>,
) -> Result<DownloadQueueAddResult, String> {
    let result = backend_download_queue_add(state, package_name.clone(), release_name).await?;

    // Mark as retried if it was added successfully
    if result.added {
        Ok(DownloadQueueAddResult {
            added: false,
            retried: true,
            package_name,
            queue_length: result.queue_length,
            reason: None,
        })
    } else {
        Ok(result)
    }
}

#[tauri::command]
#[specta]
pub async fn backend_download_pause(state: State<'_, AppState>) -> Result<(), String> {
    state.rclone.pause_downloads().await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn backend_download_resume(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.settings.get_settings().await;
    state.rclone.resume_downloads(settings.bandwidth_limit_mbps).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn backend_download_pause_item(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<bool, String> {
    state.download.lock().await.pause_item(&package_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn backend_download_resume_item(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<bool, String> {
    state.download.lock().await.resume_item(&package_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn backend_download_set_bandwidth(
    state: State<'_, AppState>,
    mbps: f64,
) -> Result<(), String> {
    state.rclone.set_bandwidth_limit(mbps).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn backend_download_location_get(state: State<'_, AppState>) -> Result<DownloadLocation, String> {
    let download_dir = state
        .download
        .lock()
        .await
        .download_dir()
        .to_path_buf();
    
    let exists = download_dir.exists();
    let file_count = if exists {
        std::fs::read_dir(&download_dir).map(|rd| rd.count()).unwrap_or(0) as u32
    } else {
        0
    };

    Ok(DownloadLocation {
        download_dir: download_dir.display().to_string(),
        path: download_dir.display().to_string(),
        exists,
        free_bytes: 100.0 * 1024.0 * 1024.0 * 1024.0, // Mock 100GB
        file_count,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_download_location_set(
    state: State<'_, AppState>,
    path: String,
) -> Result<DownloadLocation, String> {
    let patch = json!({ "download_dir": path });
    state.settings.patch_settings(patch).await.map_err(|e| e.to_string())?;
    backend_download_location_get(state).await
}

#[tauri::command]
#[specta]
pub async fn backend_download_list_local(state: State<'_, AppState>) -> Result<DownloadListLocalResult, String> {
    let download_dir = state.download.lock().await.download_dir().to_path_buf();
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&download_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let metadata = std::fs::metadata(&path).ok();
                    let size_bytes = metadata.as_ref().map(|m| m.len() as f64).unwrap_or(0.0);
                    let modified_at = metadata.as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .flatten()
                        .map(|d| d.as_secs_f64());
                    
                    files.push(LocalDownloadFile {
                        name: name.to_string(),
                        package_name: name.to_string(),
                        size_bytes,
                        modified_at,
                    });
                }
            }
        }
    }

    let count = files.len() as u32;

    Ok(DownloadListLocalResult { files, count })
}

#[tauri::command]
#[specta]
pub async fn backend_download_check_local(
    state: State<'_, AppState>,
    package_name: String,
    release_name: Option<String>,
) -> Result<DownloadCheckLocalResult, String> {
    let maybe_game = {
        let catalog = state.catalog.read().await;
        match release_name {
            Some(ref rel) => catalog
                .get_game_by_package_and_release(&package_name, rel)
                .cloned(),
            None => catalog.get_game_by_package(&package_name).cloned(),
        }
    };

    let Some(game) = maybe_game else {
        return Ok(DownloadCheckLocalResult {
            package_name,
            has_local_files: false,
            local_size_bytes: 0.0,
        });
    };

    let download = state.download.lock().await;
    let has_local_files = download.is_downloaded(&game).await;
    // TODO: Calculate actual local_size_bytes by scanning directory
    let local_size_bytes = 0.0;
    drop(download);
    
    Ok(DownloadCheckLocalResult {
        package_name,
        has_local_files,
        local_size_bytes,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_download_open_folder(
    state: State<'_, AppState>,
    _package_name: Option<String>,
) -> Result<DownloadOpenFolderResult, String> {
    let path = state.download.lock().await.download_dir().to_path_buf();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(&path).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&path).spawn();

    Ok(DownloadOpenFolderResult { opened: true })
}

#[tauri::command]
#[specta]
pub async fn backend_download_delete_files(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<DownloadDeleteResult, String> {
    let maybe_game = state.catalog.read().await.get_game_by_package(&package_name).cloned();
    let mut freed_bytes = 0.0;
    
    if let Some(game) = maybe_game {
        let dir = state.download.lock().await.get_download_dir(&game);
        if dir.exists() {
            // Calculate size before deletion
            freed_bytes = calculate_dir_size(&dir);
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
    
    Ok(DownloadDeleteResult { deleted: true, freed_bytes })
}

fn calculate_dir_size(path: &std::path::Path) -> f64 {
    let mut total_size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            } else if path.is_dir() {
                total_size += calculate_dir_size(&path) as u64;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size as f64
}

// --- Installation ---

#[tauri::command]
#[specta]
pub async fn backend_install_game(
    state: State<'_, AppState>,
    package_name: String,
    release_name: Option<String>,
) -> Result<InstallGameResult, String> {
    crate::logger::log(&format!("[IPC] backend_install_game: package={}, release={:?}", package_name, release_name));
    let catalog = state.catalog.read().await;
    let maybe_game = match release_name {
        Some(ref release) if !release.is_empty() => {
            catalog.get_game_by_package_and_release(&package_name, release).cloned()
        }
        _ => {
            catalog.get_game_by_package(&package_name).cloned()
        }
    };

    let Some(game) = maybe_game else {
        return Err(format!("No catalog game found for package: {package_name}"));
    };


    let hash_dir = state.download.lock().await.get_download_dir(&game);
    let serial = selected_serial(&state).await;
    let install_service = state.install.clone();
    let app_state = state.inner().clone();

    // Prevent concurrent installs of the same package
    if !install_service.try_start_install(&package_name).await {
        return Err(format!("Already installing {package_name}"));
    }

    let operation_id = Uuid::new_v4().to_string();

    let op_id_clone = operation_id.clone();
    let pkg_name_clone = package_name.clone();
    let release_name_clone = game.release_name.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let app_state_for_status = app_state.clone();
    let op_id_for_status = op_id_clone.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            app_state_for_status
                .push_operation_event(
                    "install.progress",
                    &op_id_for_status,
                    "install",
                    "running",
                    &msg,
                    50.0,
                )
                .await;
        }
    });

    tokio::spawn(async move {
        crate::logger::log(&format!(
            "[INSTALL] Spawned install task for package: {} ({})",
            pkg_name_clone, release_name_clone
        ));

        let password = app_state.config.fetch_config().await.ok().map(|c| c.password);

        let result = install_service
            .install_game(
                &hash_dir,
                &pkg_name_clone,
                &release_name_clone,
                serial.as_deref(),
                password,
                Some(tx),
            )
            .await;

        // Always release the install lock
        install_service.finish_install(&pkg_name_clone).await;

        match result {
            Ok(res) => {
                crate::logger::log(&format!("[INSTALL] Install service returned: success={}, message='{}'", res.success, res.message));
                let msg = if res.success { "Installation successful" } else { &res.message };
                let state = if res.success { "succeeded" } else { "failed" };
                app_state
                    .push_operation_event("install.completed", &op_id_clone, "install", state, msg, 100.0)
                    .await;
            }
            Err(e) => {
                crate::logger::log(&format!("[INSTALL] Install service returned ERR: {}", e));
                app_state
                    .push_operation_event("install.failed", &op_id_clone, "install", "failed", &e.to_string(), 0.0)
                    .await;
            }
        }
    });

    Ok(InstallGameResult {
        operation_id,
        package_name,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_install_status() -> Result<InstallStatus, String> {
    Ok(InstallStatus {
        is_installing: false,
        current_package: None,
        progress_percent: 0.0,
        status_message: String::new(),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_install_cancel() -> Result<InstallCancelResult, String> {
    Ok(InstallCancelResult { cancelled: false })
}

#[tauri::command]
#[specta]
pub async fn backend_uninstall_game(
    state: State<'_, AppState>,
    package_name: String,
    keep_obb: Option<bool>,
    keep_data: Option<bool>,
) -> Result<UninstallResult, String> {
    crate::logger::log(&format!("[IPC] backend_uninstall_game: package={}, keep_obb={:?}, keep_data={:?}", package_name, keep_obb, keep_data));
    let serial = selected_serial(&state).await;
    let result = state
        .install
        .uninstall_game(
            &package_name,
            serial.as_deref(),
            keep_obb.unwrap_or(false),
            keep_data.unwrap_or(false),
        )
        .await
        .map_err(|err| err.to_string())?;
    Ok(UninstallResult {
        uninstalled: result.success,
        package_name,
        message: Some(result.message),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_installed_apps(state: State<'_, AppState>) -> Result<InstalledAppsResult, String> {
    let serial = selected_serial(&state).await;
    let output = state.adb.shell("pm list packages --show-versioncode", serial.as_deref()).await.map_err(|e| e.to_string())?;
    let packages = AdbService::parse_packages_with_versions_output(&output.stdout);

    let catalog = state.catalog.read().await;
    let mut apps: Vec<InstalledApp> = packages
        .into_iter()
        .map(|(pkg, version)| {
            let catalog_game = catalog.get_game_by_package(&pkg);
            let in_catalog = catalog_game.is_some();
            let game_name = catalog_game.map(|g| g.game_name.clone());
            let catalog_version_code = catalog_game.map(|g| g.version_code.clone());
            let size = catalog_game.map(|g| g.size.clone());

            let mut update_available = false;
            if let (Some(installed_ver), Some(catalog_ver)) = (version.as_ref(), catalog_version_code.as_ref()) {
                let inst: i64 = installed_ver.parse().unwrap_or(0);
                let cat: i64 = catalog_ver.parse().unwrap_or(0);
                if cat > inst {
                    update_available = true;
                }
            }

            InstalledApp {
                package_name: pkg.clone(),
                app_name: game_name.clone().unwrap_or_else(|| pkg.clone()),
                version_code: version.clone().unwrap_or_default(),
                version_name: version.clone().unwrap_or_default(),
                is_system_app: false,
                install_time: None,
                last_update_time: None,
                in_catalog,
                game_name,
                catalog_version_code,
                installed_version_code: version,
                size,
                update_available,
            }
        })
        .collect();

    // Alphabetical sort by app_name
    apps.sort_by(|a, b| a.app_name.to_lowercase().cmp(&b.app_name.to_lowercase()));

    let count = apps.len() as u32;
    let has_updates = apps.iter().any(|a| a.update_available);

    Ok(InstalledAppsResult {
        apps,
        count,
        has_updates,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_installed_app_version(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<AppVersionResult, String> {
    let serial = selected_serial(&state).await;
    let output = state.adb.shell(&format!("dumpsys package {package_name}"), serial.as_deref()).await.map_err(|e| e.to_string())?;
    let packages = AdbService::parse_packages_with_versions_output(&output.stdout);
    let version_code = packages.get(&package_name).cloned().flatten().unwrap_or_default();

    Ok(AppVersionResult {
        version_code: version_code.clone(),
        version_name: version_code,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_install_local(state: State<'_, AppState>, path: String) -> Result<InstallLocalResult, String> {
    let serial = selected_serial(&state).await;
    let result = state.adb.install_apk(&path, serial.as_deref()).await.map_err(|e| e.to_string())?;
    Ok(InstallLocalResult {
        success: result.success(),
        message: result.output(),
    })
}

// --- Device Management ---

#[tauri::command]
#[specta]
pub async fn backend_device_state(state: State<'_, AppState>) -> Result<DeviceState, String> {
    let devices = state.adb.get_devices().await.map_err(|err| err.to_string())?;
    let selected = selected_serial(&state).await;

    let devices_payload: Vec<DeviceInfo> = devices
        .iter()
        .map(|device| DeviceInfo {
            serial: device.serial.clone(),
            state: device.state.clone(),
            model: device.model.clone(),
            product: device.product.clone(),
            is_selected: selected.as_deref() == Some(device.serial.as_str()),
            is_connected: device.is_connected(),
        })
        .collect();

    let status = if devices.is_empty() {
        "no_device".to_string()
    } else if devices.len() > 1 && selected.is_none() {
        "selection_required".to_string()
    } else {
        "connected".to_string()
    };

    let storage: Option<DeviceStorage> = if let Some(serial) = selected.as_deref() {
        state.adb.get_storage_info(Some(serial)).await.ok().map(|map| {
            DeviceStorage {
                total_mb: map.get("total_mb").copied().unwrap_or(0) as f64,
                used_mb: map.get("used_mb").copied().unwrap_or(0) as f64,
                free_mb: map.get("free_mb").copied().unwrap_or(0) as f64,
            }
        })
    } else {
        None
    };

    let battery: Option<DeviceBattery> = if let Some(serial) = selected.as_deref() {
        state.adb.get_battery_info(Some(serial)).await.ok().map(|map| {
            DeviceBattery {
                level_percent: map.get("level_percent").and_then(|v| v.parse().ok()),
                status: map.get("status").cloned().unwrap_or_default(),
                is_charging: map.get("is_charging").map(|v| v == "true").unwrap_or(false),
                temperature_c: map.get("temperature_c").and_then(|v| v.parse().ok()),
            }
        })
    } else {
        None
    };

    Ok(DeviceState {
        status,
        status_message: if devices.is_empty() { "No device connected".to_string() } else { "Device connected".to_string() },
        troubleshooting: if devices.is_empty() { "Please connect your Quest via USB.".to_string() } else { "No issues detected.".to_string() },
        can_download: true,
        can_install: selected.is_some(),
        download_only_mode: selected.is_none(),
        selected_serial: selected.clone(),
        selection_source: "manual".to_string(),
        devices: devices_payload,
        storage,
        battery,
        wireless: DeviceWirelessState {
            saved_endpoint: None,
            auto_reconnect_enabled: false,
            last_attempt_at: None,
            last_endpoint: None,
            last_status: None,
            last_error: None,
        },
        keep_awake: DeviceKeepAwake {
            enabled: false,
            interval_seconds: 30,
            active_count: 0,
            active_operation_ids: vec![],
        },
    })
}

#[tauri::command]
#[specta]
pub async fn backend_select_device(
    state: State<'_, AppState>,
    serial: String,
) -> Result<DeviceState, String> {
    *state.selected_serial.write().await = Some(serial);
    backend_device_state(state).await
}

#[tauri::command]
#[specta]
pub async fn backend_clear_device_selection(state: State<'_, AppState>) -> Result<DeviceState, String> {
    *state.selected_serial.write().await = None;
    backend_device_state(state).await
}

#[tauri::command]
#[specta]
pub async fn backend_wireless_connect(
    state: State<'_, AppState>,
    endpoint: Option<String>,
    save_endpoint: Option<bool>,
) -> Result<WirelessConnectResult, String> {
    let endpoint = endpoint.unwrap_or_default();
    let result = state
        .adb
        .connect_wireless(&endpoint)
        .await
        .map_err(|err| err.to_string())?;

    if save_endpoint.unwrap_or(false) {
        let patch = json!({
            "ip_address": endpoint,
            "wireless_adb": true,
        });
        let _ = state.settings.patch_settings(patch).await;
    }

    Ok(WirelessConnectResult {
        connected: result.success(),
        endpoint,
        message: result.output(),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_wireless_disconnect(
    state: State<'_, AppState>,
    endpoint: Option<String>,
) -> Result<WirelessDisconnectResult, String> {
    let result = state
        .adb
        .disconnect_wireless(endpoint.as_deref())
        .await
        .map_err(|err| err.to_string())?;
    Ok(WirelessDisconnectResult {
        disconnected: result.success(),
        message: result.output(),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_wireless_reconnect(state: State<'_, AppState>) -> Result<WirelessReconnectResult, String> {
    let settings = state.settings.get_settings().await;
    if !settings.ip_address.is_empty() {
        let endpoint = settings.ip_address.clone();
        let result = state
            .adb
            .connect_wireless(&endpoint)
            .await
            .map_err(|err| err.to_string())?;
        Ok(WirelessReconnectResult {
            reconnected: result.success(),
            endpoint: Some(endpoint),
            message: result.output(),
        })
    } else {
        Ok(WirelessReconnectResult {
            reconnected: false,
            endpoint: None,
            message: "No saved IP address".to_string(),
        })
    }
}

// --- Events ---

#[tauri::command]
#[specta]
pub async fn poll_backend_events(
    state: State<'_, AppState>,
    _operation_id: Option<String>,
    limit: Option<u32>,
) -> Result<Value, String> {
    let limit = limit.unwrap_or(100) as usize;

    // Drain operation events
    let mut events_lock = state.events.lock().await;
    let count = events_lock.len().min(limit);
    let mut drained: Vec<Value> = events_lock.drain(0..count).collect();
    drop(events_lock);

    // Drain backend log messages and include them as simple log events
    let remaining = limit.saturating_sub(drained.len());
    if remaining > 0 {
        let logs = crate::logger::drain_logs(remaining);
        for msg in logs {
            drained.push(json!({
                "kind": "log",
                "message": msg
            }));
        }
    }

    Ok(json!({
        "events": drained
    }))
}

// --- Logging & Diagnostics ---

#[tauri::command]
#[specta]
pub fn frontend_log(level: String, message: String) {
    eprintln!("[JS {level}] {message}");
}

#[tauri::command]
#[specta]
pub async fn backend_log_entries() -> Result<LogEntriesResult, String> {
    Ok(LogEntriesResult { entries: vec![] })
}

#[tauri::command]
#[specta]
pub async fn backend_log_export() -> Result<LogExportResult, String> {
    Ok(LogExportResult {
        exported: true,
        path: None,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_log_reset() -> Result<LogResetResult, String> {
    Ok(LogResetResult { reset: true })
}

#[tauri::command]
#[specta]
pub async fn backend_log_upload_payload() -> Result<LogUploadPayload, String> {
    Ok(LogUploadPayload {
        payload: String::new(),
    })
}

// --- Window State ---

#[tauri::command]
#[specta]
pub async fn backend_window_state_get(state: State<'_, AppState>) -> Result<WindowState, String> {
    let settings = state.settings.get_settings().await;
    Ok(WindowState {
        width: settings.window_width,
        height: settings.window_height,
        x: settings.window_x,
        y: settings.window_y,
        maximized: settings.window_maximized,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_window_state_set(
    state: State<'_, AppState>,
    patch: Value,
) -> Result<WindowStateSetResult, String> {
    state.settings.patch_settings(patch).await.map_err(|e| e.to_string())?;
    Ok(WindowStateSetResult { saved: true })
}

// --- Other Stubs ---

#[tauri::command]
#[specta]
pub async fn start_backend_operation() -> Result<OperationStatus, String> {
    let operation_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp() as f64;
    Ok(OperationStatus {
        operation_id,
        operation: "demo".to_string(),
        state: "running".to_string(),
        state_version: 1,
        state_history: vec![OperationStateHistoryEntry {
            version: 1,
            state: "running".to_string(),
            entered_at: now,
            reason: "Started".to_string(),
        }],
        progress: OperationProgress {
            percent: 0.0,
            completed_steps: 0,
            total_steps: 10,
        },
        cancel_requested: false,
        cancel_requested_at: None,
        terminal: false,
        terminal_at: None,
        keep_awake: Some(OperationKeepAwake {
            enabled: false,
            interval_seconds: 30,
            ticks_sent: 0,
            last_sent_at: None,
        }),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_operation_status(operation_id: String) -> Result<OperationStatus, String> {
    let now = Utc::now().timestamp() as f64;
    Ok(OperationStatus {
        operation_id,
        operation: "unknown".to_string(),
        state: "succeeded".to_string(),
        state_version: 1,
        state_history: vec![OperationStateHistoryEntry {
            version: 1,
            state: "succeeded".to_string(),
            entered_at: now,
            reason: "Operation completed".to_string(),
        }],
        progress: OperationProgress {
            percent: 100.0,
            completed_steps: 10,
            total_steps: 10,
        },
        cancel_requested: false,
        cancel_requested_at: None,
        terminal: true,
        terminal_at: Some(now),
        keep_awake: Some(OperationKeepAwake {
            enabled: false,
            interval_seconds: 30,
            ticks_sent: 0,
            last_sent_at: None,
        }),
    })
}

#[tauri::command]
#[specta]
pub async fn cancel_backend_operation(operation_id: String) -> Result<OperationStatus, String> {
    let now = Utc::now().timestamp() as f64;
    Ok(OperationStatus {
        operation_id,
        operation: "unknown".to_string(),
        state: "cancelled".to_string(),
        state_version: 2,
        state_history: vec![
            OperationStateHistoryEntry {
                version: 1,
                state: "running".to_string(),
                entered_at: now - 10.0,
                reason: "Started".to_string(),
            },
            OperationStateHistoryEntry {
                version: 2,
                state: "cancelled".to_string(),
                entered_at: now,
                reason: "User requested cancellation".to_string(),
            },
        ],
        progress: OperationProgress {
            percent: 50.0,
            completed_steps: 5,
            total_steps: 10,
        },
        cancel_requested: true,
        cancel_requested_at: Some(now),
        terminal: true,
        terminal_at: Some(now),
        keep_awake: Some(OperationKeepAwake {
            enabled: false,
            interval_seconds: 30,
            ticks_sent: 0,
            last_sent_at: None,
        }),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_detect_updates() -> Result<DetectUpdatesResult, String> {
    Ok(DetectUpdatesResult {
        has_updates: false,
        updates: vec![],
        count: 0,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_update_app(
    _state: State<'_, AppState>,
    package_name: String,
    release_name: Option<String>,
) -> Result<UpdateAppResult, String> {
    crate::logger::log(&format!("[IPC] backend_update_app: package={}, release={:?}", package_name, release_name));
    let operation_id = Uuid::new_v4().to_string();
    Ok(UpdateAppResult {
        operation_id,
        package_name,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_backup_app(
    _state: State<'_, AppState>,
    package_name: String,
) -> Result<BackupAppResult, String> {
    let operation_id = Uuid::new_v4().to_string();
    Ok(BackupAppResult {
        operation_id,
        package_name,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_restore_app(
    _state: State<'_, AppState>,
    package_name: String,
) -> Result<RestoreAppResult, String> {
    let operation_id = Uuid::new_v4().to_string();
    Ok(RestoreAppResult {
        operation_id,
        package_name,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_bulk_backup(
    _state: State<'_, AppState>,
    package_names: Vec<String>,
) -> Result<BulkBackupResult, String> {
    let operation_id = Uuid::new_v4().to_string();
    Ok(BulkBackupResult {
        operation_id,
        app_count: package_names.len() as u32,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_bulk_restore(
    _state: State<'_, AppState>,
    backup_paths: Vec<String>,
) -> Result<BulkRestoreResult, String> {
    let operation_id = Uuid::new_v4().to_string();
    Ok(BulkRestoreResult {
        operation_id,
        backup_count: backup_paths.len() as u32,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_extract_apk(
    _state: State<'_, AppState>,
    package_name: String,
) -> Result<OperationResult, String> {
    Ok(OperationResult {
        success: true,
        message: Some(format!("Extracted APK for {}", package_name)),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_list_backups(
    _state: State<'_, AppState>,
) -> Result<ListBackupsResult, String> {
    Ok(ListBackupsResult { backups: vec![] })
}

#[tauri::command]
#[specta]
pub async fn backend_delete_backup(
    _state: State<'_, AppState>,
    backup_path: String,
) -> Result<DeleteBackupResult, String> {
    Ok(DeleteBackupResult {
        deleted: true,
        backup_path,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_media_list(
    _state: State<'_, AppState>,
) -> Result<MediaListResult, String> {
    Ok(MediaListResult {
        files: vec![],
        total_size: 0.0,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_media_transfer(
    _state: State<'_, AppState>,
    _source_path: String,
    destination_path: String,
) -> Result<MediaTransferResult, String> {
    Ok(MediaTransferResult {
        transferred: true,
        bytes_transferred: 0.0,
        destination_path,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_performance_profile_get(
    state: State<'_, AppState>,
) -> Result<PerformanceProfileResult, String> {
    let settings = state.settings.get_settings().await;
    Ok(PerformanceProfileResult {
        profile: PerformanceProfile {
            profile_name: "default".to_string(),
            cpu_limit_percent: settings.performance_cpu_level as u32 * 25,
            memory_limit_mb: 2048,
            priority: "normal".to_string(),
        },
    })
}

#[tauri::command]
#[specta]
pub async fn backend_performance_profile_set(
    _state: State<'_, AppState>,
    _profile: PerformanceProfile,
) -> Result<PerformanceProfileSetResult, String> {
    Ok(PerformanceProfileSetResult { saved: true })
}

#[tauri::command]
#[specta]
pub async fn backend_adb_console_execute(
    state: State<'_, AppState>,
    command: String,
) -> Result<AdbConsoleExecuteResult, String> {
    let serial = selected_serial(&state).await;
    let output = state
        .adb
        .shell(&command, serial.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    let success = output.success();
    Ok(AdbConsoleExecuteResult {
        success,
        output: output.stdout,
        exit_code: if success { 0 } else { 1 },
    })
}

#[tauri::command]
#[specta]
pub async fn backend_adb_console_history(
    _state: State<'_, AppState>,
) -> Result<AdbConsoleHistoryResult, String> {
    Ok(AdbConsoleHistoryResult { commands: vec![] })
}

#[tauri::command]
#[specta]
pub async fn backend_new_apps_discovery(
    _state: State<'_, AppState>,
) -> Result<NewAppsDiscoveryResult, String> {
    Ok(NewAppsDiscoveryResult {
        new_apps: vec![],
        count: 0,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_donation_metadata_build(
    _state: State<'_, AppState>,
) -> Result<DonationMetadataResult, String> {
    Ok(DonationMetadataResult {
        metadata: DonationMetadata {
            donation_url: String::new(),
            qr_code_data: String::new(),
        },
    })
}

#[tauri::command]
#[specta]
pub async fn backend_privacy_status(
    state: State<'_, AppState>,
) -> Result<PrivacyStatus, String> {
    let settings = state.settings.get_settings().await;
    Ok(PrivacyStatus {
        uuid: settings.diagnostics_uuid,
        temp_dir_size_bytes: 0.0,
        can_cleanup: true,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_privacy_uuid_rotate(
    _state: State<'_, AppState>,
) -> Result<PrivacyUuidRotateResult, String> {
    let new_uuid = Uuid::new_v4().to_string();
    Ok(PrivacyUuidRotateResult {
        rotated: true,
        new_uuid,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_privacy_temp_cleanup(
    _state: State<'_, AppState>,
) -> Result<PrivacyTempCleanupResult, String> {
    Ok(PrivacyTempCleanupResult {
        cleaned: true,
        freed_bytes: 0.0,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_offline_mode_get(
    state: State<'_, AppState>,
) -> Result<OfflineModeStatus, String> {
    let settings = state.settings.get_settings().await;
    Ok(OfflineModeStatus {
        enabled: settings.offline_mode,
        last_sync_at: None,
        cache_valid: true,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_offline_mode_set(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<OfflineModeSetResult, String> {
    let patch = json!({ "offline_mode": enabled });
    state.settings.patch_settings(patch).await.map_err(|e| e.to_string())?;
    Ok(OfflineModeSetResult { enabled })
}

#[tauri::command]
#[specta]
pub async fn backend_crash_check(
    _state: State<'_, AppState>,
) -> Result<CrashInfo, String> {
    Ok(CrashInfo {
        has_crash: false,
        crash_id: None,
        timestamp: None,
        error_message: None,
    })
}

#[tauri::command]
#[specta]
pub async fn backend_crash_report(
    _state: State<'_, AppState>,
    crash_id: String,
) -> Result<CrashReportResult, String> {
    Ok(CrashReportResult {
        reported: true,
        report_id: Some(crash_id),
    })
}

#[tauri::command]
#[specta]
pub async fn backend_crash_dismiss(
    _state: State<'_, AppState>,
    _crash_id: String,
) -> Result<CrashDismissResult, String> {
    Ok(CrashDismissResult { dismissed: true })
}

pub fn register_invoke_handler(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        backend_ready_state,
        backend_recover,
        shutdown_backend,
        backend_get_settings,
        backend_update_settings,
        backend_catalog_status,
        backend_catalog_load_cache,
        backend_catalog_sync,
        backend_catalog_search,
        search_youtube_trailer,
        backend_catalog_game_detail,
        backend_catalog_game_versions,
        backend_catalog_thumbnail_path,
        backend_catalog_note,
        backend_catalog_library,
        backend_favorites_toggle,
        backend_download_queue_status,
        backend_download_queue_add,
        backend_download_queue_remove,
        backend_download_queue_reorder,
        backend_download_start_processing,
        backend_download_cancel,
        backend_download_retry,
        backend_download_pause,
        backend_download_resume,
        backend_download_set_bandwidth,
        backend_download_location_get,
        backend_download_location_set,
        backend_download_list_local,
        backend_download_check_local,
        backend_download_open_folder,
        backend_download_delete_files,
        backend_install_game,
        backend_install_status,
        backend_install_cancel,
        backend_uninstall_game,
        backend_installed_apps,
        backend_installed_app_version,
        backend_install_local,
        backend_device_state,
        backend_select_device,
        backend_clear_device_selection,
        backend_wireless_connect,
        backend_wireless_disconnect,
        backend_wireless_reconnect,
        poll_backend_events,
        frontend_log,
        backend_log_entries,
        backend_log_export,
        backend_log_reset,
        backend_log_upload_payload,
        backend_window_state_get,
        backend_window_state_set,
        start_backend_operation,
        backend_operation_status,
        cancel_backend_operation,
        backend_detect_updates,
        backend_update_app,
        backend_backup_app,
        backend_restore_app,
        backend_bulk_backup,
        backend_bulk_restore,
        backend_extract_apk,
        backend_list_backups,
        backend_delete_backup,
        backend_media_list,
        backend_media_transfer,
        backend_performance_profile_get,
        backend_performance_profile_set,
        backend_adb_console_execute,
        backend_adb_console_history,
        backend_new_apps_discovery,
        backend_donation_metadata_build,
        backend_privacy_status,
        backend_privacy_uuid_rotate,
        backend_privacy_temp_cleanup,
        backend_offline_mode_get,
        backend_offline_mode_set,
        backend_crash_check,
        backend_crash_report,
        backend_crash_dismiss,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_bindings() {
        use crate::models::game::Game;
        use crate::models::settings::Settings;
        use crate::models::config::PublicConfig;
        use crate::models::responses::*;

        tauri_specta::Builder::<tauri::Wry>::new()
            .typ::<Game>()
            .typ::<Settings>()
            .typ::<PublicConfig>()
            .typ::<BackendReadyState>()
            .typ::<BackendRecoverResult>()
            .typ::<SettingsResponse>()
            .typ::<SettingsUpdateResponse>()
            .typ::<CatalogStatus>()
            .typ::<CatalogSyncResult>()
            .typ::<CatalogSearchGame>()
            .typ::<CatalogSearchResult>()
            .typ::<CatalogGameDetail>()
            .typ::<CatalogGameVersions>()
            .typ::<CatalogThumbnailPath>()
            .typ::<CatalogNote>()
            .typ::<LibraryGame>()
            .typ::<LibraryResult>()
            .typ::<FavoritesToggleResult>()
            .typ::<DownloadQueueItem>()
            .typ::<DownloadQueueStatus>()
            .typ::<DownloadQueueAddResult>()
            .typ::<DownloadQueueRemoveResult>()
            .typ::<DownloadStartResult>()
            .typ::<DownloadCancelResult>()
            .typ::<DownloadRetryResult>()
            .typ::<DownloadLocation>()
            .typ::<LocalDownloadFile>()
            .typ::<DownloadListLocalResult>()
            .typ::<DownloadCheckLocalResult>()
            .typ::<DownloadOpenFolderResult>()
            .typ::<DownloadDeleteResult>()
            .typ::<InstallGameResult>()
            .typ::<InstallStatus>()
            .typ::<InstallCancelResult>()
            .typ::<InstallLocalResult>()
            .typ::<InstalledApp>()
            .typ::<InstalledAppsResult>()
            .typ::<AppVersionResult>()
            .typ::<UninstallResult>()
            .typ::<DeviceInfo>()
            .typ::<DeviceStorage>()
            .typ::<DeviceBattery>()
            .typ::<DeviceWirelessState>()
            .typ::<DeviceKeepAwake>()
            .typ::<DeviceState>()
            .typ::<DeviceSelectResult>()
            .typ::<DeviceClearSelectionResult>()
            .typ::<WirelessConnectResult>()
            .typ::<WirelessDisconnectResult>()
            .typ::<WirelessReconnectResult>()
            .typ::<LogEntry>()
            .typ::<LogEntriesResult>()
            .typ::<LogExportResult>()
            .typ::<LogResetResult>()
            .typ::<LogUploadPayload>()
            .typ::<WindowState>()
            .typ::<WindowStateResult>()
            .typ::<WindowStateSetResult>()
            .typ::<OperationStateHistoryEntry>()
            .typ::<OperationProgress>()
            .typ::<OperationKeepAwake>()
            .typ::<OperationStatus>()
            .typ::<BackupInfo>()
            .typ::<ListBackupsResult>()
            .typ::<BackupAppResult>()
            .typ::<RestoreAppResult>()
            .typ::<DeleteBackupResult>()
            .typ::<BulkBackupResult>()
            .typ::<BulkRestoreResult>()
            .typ::<UpdateInfo>()
            .typ::<DetectUpdatesResult>()
            .typ::<UpdateAppResult>()
            .typ::<MediaFile>()
            .typ::<MediaListResult>()
            .typ::<MediaTransferResult>()
            .typ::<PerformanceProfile>()
            .typ::<PerformanceProfileResult>()
            .typ::<PerformanceProfileSetResult>()
            .typ::<AdbConsoleExecuteResult>()
            .typ::<AdbConsoleHistoryResult>()
            .typ::<NewAppInfo>()
            .typ::<NewAppsDiscoveryResult>()
            .typ::<DonationMetadata>()
            .typ::<DonationMetadataResult>()
            .typ::<PrivacyStatus>()
            .typ::<PrivacyUuidRotateResult>()
            .typ::<PrivacyTempCleanupResult>()
            .typ::<OfflineModeStatus>()
            .typ::<OfflineModeSetResult>()
            .typ::<CrashInfo>()
            .typ::<CrashReportResult>()
            .typ::<CrashDismissResult>()
            .commands(tauri_specta::collect_commands![
                backend_ready_state,
                backend_recover,
                shutdown_backend,
                backend_get_settings,
                backend_update_settings,
                backend_catalog_status,
                backend_catalog_load_cache,
                backend_catalog_sync,
                backend_catalog_search,
                search_youtube_trailer,
                backend_catalog_game_detail,
                backend_catalog_game_versions,
                backend_catalog_thumbnail_path,
                backend_catalog_note,
                backend_catalog_library,
                backend_favorites_toggle,
                backend_download_queue_status,
                backend_download_queue_add,
                backend_download_queue_remove,
                backend_download_queue_reorder,
                backend_download_start_processing,
                backend_download_cancel,
                backend_download_retry,
                backend_download_pause,
                backend_download_resume,
                backend_download_set_bandwidth,
                backend_download_location_get,
                backend_download_location_set,
                backend_download_list_local,
                backend_download_check_local,
                backend_download_open_folder,
                backend_download_delete_files,
                backend_install_game,
                backend_install_status,
                backend_install_cancel,
                backend_uninstall_game,
                backend_installed_apps,
                backend_installed_app_version,
                backend_install_local,
                backend_device_state,
                backend_select_device,
                backend_clear_device_selection,
                backend_wireless_connect,
                backend_wireless_disconnect,
                backend_wireless_reconnect,
                poll_backend_events,
                frontend_log,
                backend_log_entries,
                backend_log_export,
                backend_log_reset,
                backend_log_upload_payload,
                backend_window_state_get,
                backend_window_state_set,
                start_backend_operation,
                backend_operation_status,
                cancel_backend_operation,
                backend_detect_updates,
                backend_update_app,
                backend_backup_app,
                backend_restore_app,
                backend_bulk_backup,
                backend_bulk_restore,
                backend_extract_apk,
                backend_list_backups,
                backend_delete_backup,
                backend_media_list,
                backend_media_transfer,
                backend_performance_profile_get,
                backend_performance_profile_set,
                backend_adb_console_execute,
                backend_adb_console_history,
                backend_new_apps_discovery,
                backend_donation_metadata_build,
                backend_privacy_status,
                backend_privacy_uuid_rotate,
                backend_privacy_temp_cleanup,
                backend_offline_mode_get,
                backend_offline_mode_set,
                backend_crash_check,
                backend_crash_report,
                backend_crash_dismiss,
            ])
            .export(
                specta_typescript::Typescript::default(),
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../src/bindings.ts"),
            )
            .expect("Failed to export bindings");
    }
}
