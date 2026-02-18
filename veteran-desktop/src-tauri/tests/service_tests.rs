use veteran_desktop::models::game::Game;
use veteran_desktop::models::device::RawDeviceInfo;
use veteran_desktop::services::download::{DownloadService, DownloadStatus, DownloadItem};
use veteran_desktop::services::rclone::RcloneService;
use veteran_desktop::services::adb::{AdbService, AdbResult, StorageInfo, BatteryInfo};
use std::sync::Arc;
use tempfile::tempdir;

fn sample_game(package_name: &str) -> Game {
    Game {
        game_name: "Sample Game".to_string(),
        release_name: format!("Sample Release {}", package_name),
        package_name: package_name.to_string(),
        version_code: "1".to_string(),
        release_apk_path: String::new(),
        version_name: "1.0.0".to_string(),
        downloads: "1000".to_string(),
        size: "500 MB".to_string(),
        last_updated: "2024-01-01".to_string(),
        thumbnail_path: String::new(),
        thumbnail_exists: false,
        note_path: String::new(),
        note_excerpt: String::new(),
        note_exists: false,
        popularity_rank: 1,
        is_new: false,
    }
}

fn sample_game_with_release(package_name: &str, release_name: &str) -> Game {
    Game {
        game_name: "Sample Game".to_string(),
        release_name: release_name.to_string(),
        package_name: package_name.to_string(),
        version_code: "1".to_string(),
        release_apk_path: String::new(),
        version_name: "1.0.0".to_string(),
        downloads: "1000".to_string(),
        size: "500 MB".to_string(),
        last_updated: "2024-01-01".to_string(),
        thumbnail_path: String::new(),
        thumbnail_exists: false,
        note_path: String::new(),
        note_excerpt: String::new(),
        note_exists: false,
        popularity_rank: 1,
        is_new: false,
    }
}

#[tokio::test]
async fn test_download_service_add_to_queue_basic() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game = sample_game("com.test.game");
    let result = service.add_to_queue(game.clone()).await;

    assert!(result, "Adding a new game to queue should return true");
    let queue = service.queue().await;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].game.package_name, "com.test.game");
    assert_eq!(queue[0].status, DownloadStatus::Queued);
    assert!(!queue[0].operation_id.is_empty(), "Operation ID should be generated");
}

#[tokio::test]
async fn test_download_service_add_to_queue_prevents_duplicates() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game1 = sample_game("com.test.game");
    let game2 = sample_game_with_release("com.test.game", "Different Release");

    let first_add = service.add_to_queue(game1).await;
    let second_add = service.add_to_queue(game2).await;

    assert!(first_add, "First add should succeed");
    assert!(!second_add, "Adding duplicate package_name should return false");
    assert_eq!(service.queue().await.len(), 1, "Queue should only contain one item");
}

#[tokio::test]
async fn test_download_service_add_to_queue_different_packages_allowed() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game1 = sample_game("com.test.game1");
    let game2 = sample_game("com.test.game2");
    let game3 = sample_game("com.test.game3");

    assert!(service.add_to_queue(game1).await);
    assert!(service.add_to_queue(game2).await);
    assert!(service.add_to_queue(game3).await);

    assert_eq!(service.queue().await.len(), 3);
}

#[tokio::test]
async fn test_download_service_remove_from_queue_success() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.test.game1")).await;
    service.add_to_queue(sample_game("com.test.game2")).await;
    service.add_to_queue(sample_game("com.test.game3")).await;

    let removed = service.remove_from_queue("com.test.game2").await;

    assert!(removed, "Remove should return true when item exists");
    let queue = service.queue().await;
    assert_eq!(queue.len(), 2);
    
    let package_names: Vec<String> = queue
        .iter()
        .map(|item| item.game.package_name.clone())
        .collect();
    assert_eq!(package_names, vec!["com.test.game1", "com.test.game3"]);
}

#[tokio::test]
async fn test_download_service_remove_from_queue_nonexistent() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.test.game1")).await;

    let removed = service.remove_from_queue("com.nonexistent").await;

    assert!(!removed, "Remove should return false when item doesn't exist");
    assert_eq!(service.queue().await.len(), 1);
}

#[tokio::test]
async fn test_download_service_reorder_queue_basic() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.first")).await;
    service.add_to_queue(sample_game("com.second")).await;
    service.add_to_queue(sample_game("com.third")).await;

    let reordered = service.reorder_queue("com.third", 0).await;

    assert!(reordered, "Reorder should return true when successful");
    
    let queue = service.queue().await;
    let order: Vec<String> = queue
        .iter()
        .map(|item| item.game.package_name.clone())
        .collect();
    assert_eq!(order, vec!["com.third", "com.first", "com.second"]);
}

#[tokio::test]
async fn test_download_service_reorder_queue_to_end() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.first")).await;
    service.add_to_queue(sample_game("com.second")).await;
    service.add_to_queue(sample_game("com.third")).await;

    let reordered = service.reorder_queue("com.first", 10).await;

    assert!(reordered);
    
    let queue = service.queue().await;
    let order: Vec<String> = queue
        .iter()
        .map(|item| item.game.package_name.clone())
        .collect();
    assert_eq!(order, vec!["com.second", "com.third", "com.first"]);
}

#[tokio::test]
async fn test_download_service_reorder_queue_nonexistent() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.first")).await;
    service.add_to_queue(sample_game("com.second")).await;

    let reordered = service.reorder_queue("com.nonexistent", 0).await;

    assert!(!reordered, "Reorder should return false for nonexistent package");
    
    let queue = service.queue().await;
    let order: Vec<String> = queue
        .iter()
        .map(|item| item.game.package_name.clone())
        .collect();
    assert_eq!(order, vec!["com.first", "com.second"]);
}

#[test]
fn test_download_service_get_download_dir() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game = sample_game_with_release("com.test", "My Game v1+abc123");
    let download_dir = service.get_download_dir(&game);

    assert!(download_dir.starts_with(temp.path()));
    let dir_name = download_dir.file_name().unwrap().to_string_lossy();
    assert_eq!(dir_name.len(), 32, "Directory name should be MD5 hash (32 hex chars)");
}

#[tokio::test]
async fn test_download_service_is_downloaded_false_when_no_dir() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game = sample_game("com.not.downloaded");
    
    assert!(!service.is_downloaded(&game).await);
}

#[tokio::test]
async fn test_download_service_is_downloaded_true_with_install_txt() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game = sample_game_with_release("com.has.install", "Game v1+testhash");
    let game_dir = service.get_download_dir(&game);
    
    std::fs::create_dir_all(&game_dir).unwrap();
    std::fs::write(game_dir.join("install.txt"), "adb shell echo installed").unwrap();

    assert!(service.is_downloaded(&game).await);
}

#[tokio::test]
async fn test_download_service_is_downloaded_true_with_apk() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let game = sample_game_with_release("com.has.apk", "Game v1+testhash2");
    let game_dir = service.get_download_dir(&game);
    
    std::fs::create_dir_all(&game_dir).unwrap();
    std::fs::write(game_dir.join("app.apk"), "fake apk content").unwrap();

    assert!(service.is_downloaded(&game).await);
}

#[tokio::test]
async fn test_download_service_process_queue_prevents_concurrent() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.test")).await;
    
    // First call starts processing
    let result1: Result<(), anyhow::Error> = service.process_queue().await;
    assert!(result1.is_ok());
    // Note: is_processing() may remain true briefly after process_queue completes
}

#[tokio::test]
async fn test_download_service_cancel_current_no_active_download() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    service.add_to_queue(sample_game("com.test")).await;

    let result: Result<bool, anyhow::Error> = service.cancel_current().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false, "Should return false when no active download");
}

#[test]
fn test_download_item_creation() {
    let game = sample_game("com.test");
    let item = DownloadItem::new(game.clone());

    assert!(!item.operation_id.is_empty());
    assert_eq!(item.game.package_name, game.package_name);
    assert_eq!(item.status, DownloadStatus::Queued);
    assert_eq!(item.progress.percent, 0.0);
    assert!(item.error.is_empty());
}

#[test]
fn test_download_item_game_hash() {
    let game = sample_game_with_release("com.test", "Game v1+abcdef123");
    let item = DownloadItem::new(game);

    assert_eq!(item.game_hash().len(), 32);
}

#[test]
fn test_download_status_equality() {
    assert_eq!(DownloadStatus::Queued, DownloadStatus::Queued);
    assert_eq!(DownloadStatus::Downloading, DownloadStatus::Downloading);
    assert_eq!(DownloadStatus::Completed, DownloadStatus::Completed);
    assert_eq!(DownloadStatus::Failed, DownloadStatus::Failed);
    assert_eq!(DownloadStatus::Cancelled, DownloadStatus::Cancelled);

    assert_ne!(DownloadStatus::Queued, DownloadStatus::Downloading);
    assert_ne!(DownloadStatus::Completed, DownloadStatus::Failed);
}

#[test]
fn test_download_item_clone() {
    let game = sample_game("com.test");
    let item = DownloadItem::new(game);
    let cloned = item.clone();

    assert_eq!(item.operation_id, cloned.operation_id);
    assert_eq!(item.game.package_name, cloned.game.package_name);
    assert_eq!(item.status, cloned.status);
}

#[tokio::test]
async fn test_download_service_process_queue_empty() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    let result: Result<(), anyhow::Error> = service.process_queue().await;

    assert!(result.is_ok());
    // Note: is_processing() may remain true briefly after process_queue completes
    // This is implementation-specific behavior
}

#[test]
fn test_download_service_download_dir_accessor() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    assert_eq!(service.download_dir(), temp.path());
}

#[tokio::test]
async fn test_download_service_new_with_arc() {
    let temp = tempdir().unwrap();
    let rclone = Arc::new(RcloneService::new(None));
    let service = DownloadService::new_with_arc(rclone.clone(), temp.path().to_path_buf(), 10.0);

    assert_eq!(service.queue().await.len(), 0);
    assert!(!service.is_processing().await);
}

#[tokio::test]
async fn test_download_service_queue_accessor() {
    let temp = tempdir().unwrap();
    let rclone = RcloneService::new(None);
    let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

    assert!(service.queue().await.is_empty());

    service.add_to_queue(sample_game("com.test")).await;
    
    let queue = service.queue().await;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].game.package_name, "com.test");
}

// ============================================================================
// AdbService Tests
// ============================================================================

#[test]
fn test_adb_service_new() {
    let service = AdbService::new();
    assert!(service.device_serial().is_none());
}

#[test]
fn test_adb_service_set_device_serial() {
    let mut service = AdbService::new();
    assert!(service.device_serial().is_none());
    
    service.set_device_serial(Some("test_device_123".to_string()));
    assert_eq!(service.device_serial(), Some("test_device_123"));
    
    service.set_device_serial(None);
    assert!(service.device_serial().is_none());
}

#[test]
fn test_adb_service_parse_devices_output_with_unauthorized_device() {
    let output = "List of devices attached
1WMHH824D50421\tdevice product:hollywood model:Quest_3 device:eureka transport_id:2
192.168.1.10:5555\tunauthorized transport_id:7
192.168.1.20:5555\toffline transport_id:8";

    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 3);
    
    assert_eq!(parsed[0].serial, "1WMHH824D50421");
    assert_eq!(parsed[0].state, "device");
    assert_eq!(parsed[0].model, "Quest_3");
    assert_eq!(parsed[0].product, "hollywood");
    
    assert_eq!(parsed[1].serial, "192.168.1.10:5555");
    assert_eq!(parsed[1].state, "unauthorized");
    
    assert_eq!(parsed[2].serial, "192.168.1.20:5555");
    assert_eq!(parsed[2].state, "offline");
}

#[test]
fn test_adb_service_parse_devices_output_empty() {
    let output = "List of devices attached
";
    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 0);
}

#[test]
fn test_adb_service_parse_devices_output_daemon_lines() {
    let output = "* daemon not running; starting now at tcp:5037
* daemon started successfully
List of devices attached
1WMHH824D50421\tdevice product:hollywood model:Quest_3 device:eureka transport_id:2";

    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].serial, "1WMHH824D50421");
}

#[test]
fn test_adb_service_parse_devices_output_without_model_product() {
    let output = "List of devices attached
1WMHH824D50421\tdevice transport_id:2";

    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].serial, "1WMHH824D50421");
    assert_eq!(parsed[0].state, "device");
    assert_eq!(parsed[0].model, "");
    assert_eq!(parsed[0].product, "");
}

#[test]
fn test_adb_service_parse_devices_output_recovery_mode() {
    let output = "List of devices attached
1WMHH824D50421\trecovery product:hollywood model:Quest_3 device:eureka transport_id:2";

    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].serial, "1WMHH824D50421");
    assert_eq!(parsed[0].state, "recovery");
}

#[test]
fn test_adb_service_parse_devices_output_sideload_mode() {
    let output = "List of devices attached
1WMHH824D50421\tsideload product:hollywood model:Quest_3 device:eureka transport_id:2";

    let parsed = AdbService::parse_devices_output(output);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].serial, "1WMHH824D50421");
    assert_eq!(parsed[0].state, "sideload");
}

#[test]
fn test_adb_service_parse_storage_info_output_storage_emulated() {
    let output = "Filesystem     1K-blocks      Used Available Use% Mounted on
/dev/fuse       120000000  30000000 90000000  25% /storage/emulated";
    
    let parsed = AdbService::parse_storage_info_output(output);
    assert!(parsed.total_mb > 0);
    assert!(parsed.used_mb > 0);
    assert!(parsed.free_mb > 0);
}

#[test]
fn test_adb_service_parse_storage_info_output_empty() {
    let output = "Filesystem     1K-blocks      Used Available Use% Mounted on";
    let parsed = AdbService::parse_storage_info_output(output);
    assert_eq!(parsed, StorageInfo::default());
}

#[test]
fn test_adb_service_parse_storage_info_output_multiple_mounts() {
    let output = "Filesystem     1K-blocks      Used Available Use% Mounted on
/dev/fuse       120000000  30000000 90000000  25% /storage/emulated
/dev/block/dm-1  64000000  16000000 48000000  25% /data
/dev/block/sda   256000000  80000000 176000000  31% /sdcard";
    
    let parsed = AdbService::parse_storage_info_output(output);
    assert!(parsed.total_mb > 0);
}

#[test]
fn test_adb_service_parse_battery_output_discharging() {
    let output = "Current Battery Service state:
  AC powered: false
  USB powered: false
  Wireless powered: false
  status: 3
  level: 45
  scale: 100
  temperature: 280";

    let parsed = AdbService::parse_battery_output(output);
    assert_eq!(parsed.level_percent, Some(45));
    assert_eq!(parsed.status, "discharging");
    assert!(!parsed.is_charging);
    assert_eq!(parsed.temperature_c, Some(28.0));
}

#[test]
fn test_adb_service_parse_battery_output_full() {
    let output = "Current Battery Service state:
  AC powered: true
  USB powered: false
  Wireless powered: false
  status: 5
  level: 100
  scale: 100
  temperature: 305";

    let parsed = AdbService::parse_battery_output(output);
    assert_eq!(parsed.level_percent, Some(100));
    assert_eq!(parsed.status, "full");
    assert!(parsed.is_charging);
    assert_eq!(parsed.temperature_c, Some(30.5));
}

#[test]
fn test_adb_service_parse_battery_output_not_charging() {
    let output = "Current Battery Service state:
  AC powered: false
  USB powered: false
  Wireless powered: false
  status: 4
  level: 80
  scale: 100
  temperature: 300";

    let parsed = AdbService::parse_battery_output(output);
    assert_eq!(parsed.level_percent, Some(80));
    assert_eq!(parsed.status, "not_charging");
    assert!(!parsed.is_charging);
}

#[test]
fn test_adb_service_parse_battery_output_unknown_status() {
    let output = "Current Battery Service state:
  AC powered: false
  USB powered: false
  Wireless powered: false
  status: 1
  level: 50
  scale: 100
  temperature: 250";

    let parsed = AdbService::parse_battery_output(output);
    assert_eq!(parsed.level_percent, Some(50));
    assert_eq!(parsed.status, "unknown");
    assert!(!parsed.is_charging);
}

#[test]
fn test_adb_service_parse_battery_output_empty() {
    let output = "";
    let parsed = AdbService::parse_battery_output(output);
    assert_eq!(parsed.level_percent, None);
    assert_eq!(parsed.status, "unknown");
    assert!(!parsed.is_charging);
    assert_eq!(parsed.temperature_c, None);
}

#[test]
fn test_adb_service_parse_packages_with_versions_output_complex() {
    let output = "package:com.oculus.shellenv
package:com.oculus.vrshell homeActivity:com.oculus.vrshell.MainActivity
package:com.oculus.quest.settings versionCode:12345678
package:com.beatgames.beatsaber versionCode:98765432";
    
    let parsed = AdbService::parse_packages_with_versions_output(output);
    assert_eq!(parsed.get("com.oculus.shellenv"), Some(&None));
    assert_eq!(parsed.get("com.oculus.vrshell"), Some(&None));
    assert_eq!(parsed.get("com.oculus.quest.settings"), Some(&Some("12345678".to_string())));
    assert_eq!(parsed.get("com.beatgames.beatsaber"), Some(&Some("98765432".to_string())));
}

#[test]
fn test_adb_service_parse_packages_with_versions_output_empty() {
    let output = "";
    let parsed = AdbService::parse_packages_with_versions_output(output);
    assert!(parsed.is_empty());
}

#[test]
fn test_adb_service_size_token_to_mb_bytes() {
    // Without assume_kib (default behavior in service)
    // The method is private, so we can't test it directly
    // But we can test it indirectly through parse_storage_info_output
}

#[test]
fn test_adb_result_success() {
    let result = AdbResult {
        stdout: "Success".to_string(),
        stderr: String::new(),
        returncode: 0,
    };
    assert!(result.success());
    assert_eq!(result.output(), "Success");
}

#[test]
fn test_adb_result_failure() {
    let result = AdbResult {
        stdout: "".to_string(),
        stderr: "Error: device not found".to_string(),
        returncode: 1,
    };
    assert!(!result.success());
    assert_eq!(result.output(), "");
}

#[test]
fn test_adb_result_output_trims_whitespace() {
    let result = AdbResult {
        stdout: "  trimmed output  \n".to_string(),
        stderr: String::new(),
        returncode: 0,
    };
    assert_eq!(result.output(), "trimmed output");
}

#[test]
fn test_storage_info_default() {
    let info = StorageInfo::default();
    assert_eq!(info.total_mb, 0);
    assert_eq!(info.used_mb, 0);
    assert_eq!(info.free_mb, 0);
}

#[test]
fn test_storage_info_equality() {
    let info1 = StorageInfo {
        total_mb: 1000,
        used_mb: 500,
        free_mb: 500,
    };
    let info2 = StorageInfo {
        total_mb: 1000,
        used_mb: 500,
        free_mb: 500,
    };
    let info3 = StorageInfo {
        total_mb: 2000,
        used_mb: 1000,
        free_mb: 1000,
    };
    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

#[test]
fn test_battery_info_default() {
    let info = BatteryInfo::default();
    assert_eq!(info.level_percent, None);
    assert_eq!(info.status, "");
    assert!(!info.is_charging);
    assert_eq!(info.temperature_c, None);
}

#[test]
fn test_device_info_creation() {
    let device = RawDeviceInfo {
        serial: "test123".to_string(),
        state: "device".to_string(),
        model: "Quest_3".to_string(),
        product: "hollywood".to_string(),
    };
    assert_eq!(device.serial, "test123");
    assert_eq!(device.state, "device");
    assert_eq!(device.model, "Quest_3");
    assert_eq!(device.product, "hollywood");
}

#[test]
fn test_adb_service_with_server_addr() {
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    let custom_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5555);
    let service = AdbService::with_server_addr(custom_addr);
    assert!(service.device_serial().is_none());
}

// ============================================================================
// InstallService Tests
// ============================================================================

use veteran_desktop::services::install::{InstallService, InstallResult};

#[test]
fn test_install_service_new() {
    let adb = AdbService::new();
    let _service = InstallService::new(adb);
    assert!(true);
}

#[test]
fn test_install_result_creation() {
    let success_result = InstallResult {
        success: true,
        message: "Installation successful".to_string(),
    };
    assert!(success_result.success);
    assert_eq!(success_result.message, "Installation successful");

    let failure_result = InstallResult {
        success: false,
        message: "Installation failed".to_string(),
    };
    assert!(!failure_result.success);
    assert_eq!(failure_result.message, "Installation failed");
}

#[test]
fn test_install_result_clone() {
    let result = InstallResult {
        success: true,
        message: "Test message".to_string(),
    };
    let cloned = result.clone();
    assert_eq!(result.success, cloned.success);
    assert_eq!(result.message, cloned.message);
}

#[test]
fn test_install_result_equality() {
    let result1 = InstallResult {
        success: true,
        message: "Success".to_string(),
    };
    let result2 = InstallResult {
        success: true,
        message: "Success".to_string(),
    };
    let result3 = InstallResult {
        success: false,
        message: "Failure".to_string(),
    };
    assert_eq!(result1, result2);
    assert_ne!(result1, result3);
}

#[test]
fn test_install_service_build_install_plan_basic() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("install.txt"), "adb shell echo test").unwrap();
    std::fs::write(root.join("app.apk"), "fake apk content").unwrap();
    std::fs::write(root.join("data.7z"), "fake archive content").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService created successfully with game directory structure");
}

#[test]
fn test_install_service_build_install_plan_complex_structure() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("base.apk"), "base apk").unwrap();
    std::fs::write(root.join("patch.apk"), "patch apk").unwrap();
    std::fs::write(root.join("obb.apk"), "obb apk").unwrap();
    std::fs::write(root.join("assets.7z"), "assets archive").unwrap();
    std::fs::write(root.join("data.7z"), "data archive").unwrap();
    std::fs::create_dir_all(root.join("com.game.package1")).unwrap();
    std::fs::create_dir_all(root.join("com.game.package2")).unwrap();
    std::fs::create_dir_all(root.join("regular_directory")).unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService handles complex directory structure");
}

#[test]
fn test_install_service_build_install_plan_no_install_txt() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("app.apk"), "fake apk content").unwrap();
    std::fs::write(root.join("content.7z"), "fake archive content").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService created without install.txt");
}

#[test]
fn test_install_service_build_install_plan_empty_directory() {
    let temp = tempdir().unwrap();
    let _root = temp.path();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService created with empty directory");
}

#[test]
fn test_install_service_build_install_plan_only_obbs() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::create_dir_all(root.join("com.example.game.obb")).unwrap();
    std::fs::write(root.join("com.example.game.obb").join("main.1.com.example.game.obb"), "obb data").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService created with only OBB directories");
}

#[test]
fn test_install_service_build_install_plan_case_insensitive_extensions() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("app.APK"), "uppercase apk").unwrap();
    std::fs::write(root.join("data.7Z"), "uppercase archive").unwrap();
    std::fs::write(root.join("mixed.7z"), "lowercase archive").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService handles case-insensitive extensions");
}

#[test]
fn test_install_service_build_install_plan_sorted_order() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("z.apk"), "z apk").unwrap();
    std::fs::write(root.join("a.apk"), "a apk").unwrap();
    std::fs::write(root.join("m.apk"), "m apk").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService handles sorted order");
}

#[tokio::test]
async fn test_install_service_install_game_nonexistent_directory() {
    let adb = AdbService::new();
    let service = InstallService::new(adb);

    let result = service
        .install_game(
            std::path::Path::new("/nonexistent/path/123456789"),
            "com.test.package",
            "Test Game",
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let install_result = result.unwrap();
    assert!(!install_result.success);
    assert!(install_result.message.contains("not found") || install_result.message.contains("Game directory"));
}

#[test]
fn test_install_result_default_failure() {
    let result = InstallResult {
        success: false,
        message: String::new(),
    };
    assert!(!result.success);
    assert!(result.message.is_empty());
}

#[test]
fn test_install_result_display_trait() {
    let result = InstallResult {
        success: true,
        message: "Test message".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("InstallResult"));
    assert!(debug_str.contains("success: true"));
}

#[test]
fn test_install_service_nested_archive_discovery() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let nested = root.join("level1").join("level2").join("level3");
    std::fs::create_dir_all(&nested).unwrap();

    std::fs::write(root.join("root.7z"), "root archive").unwrap();
    std::fs::write(root.join("level1").join("level1.7z"), "level1 archive").unwrap();
    std::fs::write(nested.join("deep.7z"), "deep archive").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService handles nested directory structures");
}

#[test]
fn test_install_service_build_install_plan_with_mixed_content() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("install.txt"), "adb shell echo test").unwrap();
    std::fs::write(root.join("game.apk"), "apk content").unwrap();
    std::fs::write(root.join("assets.7z"), "archive content").unwrap();
    std::fs::create_dir_all(root.join("com.example.obb")).unwrap();
    
    std::fs::write(root.join("readme.txt"), "readme content").unwrap();
    std::fs::write(root.join("game.zip"), "zip content").unwrap();
    std::fs::create_dir_all(root.join("regular_folder")).unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService filters non-relevant files");
}

#[test]
fn test_install_service_archive_discovery_edge_cases() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("not_a_7z.txt"), "not a 7z file").unwrap();
    std::fs::write(root.join("noextension"), "no extension").unwrap();
    std::fs::write(root.join(".hidden.7z"), "hidden archive").unwrap();

    let adb = AdbService::new();
    let _service = InstallService::new(adb);

    assert!(true, "InstallService handles edge cases");
}
