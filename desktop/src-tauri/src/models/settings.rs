use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(default)] // Use Default implementation for missing fields
pub struct Settings {
    #[serde(alias = "downloaddir")]
    pub download_dir: String,

    #[serde(alias = "deleted_after_install", alias = "deleteallafterinstall")]
    pub delete_after_install: bool,

    #[serde(alias = "bandwidthlimit")]
    pub bandwidth_limit_mbps: f64,

    #[serde(alias = "ipaddress")]
    pub ip_address: String,

    #[serde(alias = "wirelessadb")]
    pub wireless_adb: bool,

    #[serde(alias = "favoritedgames")]
    pub favorited_games: Vec<String>,

    #[serde(alias = "usegalleryview")]
    pub use_gallery_view: bool,

    #[serde(alias = "sortcolumn")]
    pub sort_column: i32,

    #[serde(alias = "sortascending")]
    pub sort_ascending: bool,

    #[serde(alias = "queuedgames")]
    pub queued_games: Vec<String>,

    #[serde(alias = "keepawakeduringlongops")]
    pub keep_awake_during_long_ops: bool,

    #[serde(alias = "keepawakeintervalseconds")]
    pub keep_awake_interval_seconds: i32,

    #[serde(alias = "firstrun")]
    pub first_run: bool,

    #[serde(alias = "offlinemode")]
    pub offline_mode: bool,

    #[serde(alias = "windowwidth")]
    pub window_width: i32,

    #[serde(alias = "windowheight")]
    pub window_height: i32,

    #[serde(alias = "windowx")]
    pub window_x: i32,

    #[serde(alias = "windowy")]
    pub window_y: i32,

    #[serde(alias = "windowmaximized")]
    pub window_maximized: bool,

    #[serde(alias = "diagnosticsuuid")]
    pub diagnostics_uuid: String,

    #[serde(alias = "knowncatalogpackages")]
    pub known_catalog_packages: Vec<String>,

    #[serde(alias = "performancerefreshratehz")]
    pub performance_refresh_rate_hz: i32,

    #[serde(alias = "performancecpulevel")]
    pub performance_cpu_level: i32,

    #[serde(alias = "performancegpulevel")]
    pub performance_gpu_level: i32,

    #[serde(alias = "performancetexturesize")]
    pub performance_texture_size: i32,
}

impl Default for Settings {
    fn default() -> Self {
        let download_dir = dirs::home_dir()
            .map(|d| {
                d.join("Veteran")
                    .join("Downloads")
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| String::from(""));

        Self {
            download_dir,
            delete_after_install: true,
            bandwidth_limit_mbps: 0.0,
            ip_address: String::new(),
            wireless_adb: false,
            favorited_games: Vec::new(),
            use_gallery_view: false,
            sort_column: 1,
            sort_ascending: false,
            queued_games: Vec::new(),
            keep_awake_during_long_ops: true,
            keep_awake_interval_seconds: 30,
            first_run: true,
            offline_mode: false,
            window_width: 1120,
            window_height: 760,
            window_x: -1,
            window_y: -1,
            window_maximized: false,
            diagnostics_uuid: String::new(),
            known_catalog_packages: Vec::new(),
            performance_refresh_rate_hz: 90,
            performance_cpu_level: 2,
            performance_gpu_level: 2,
            performance_texture_size: 1536,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(
            settings.download_dir.ends_with("Veteran/Downloads")
                || settings.download_dir.ends_with("Veteran\\Downloads")
        );
        assert!(settings.delete_after_install);
        assert_eq!(settings.window_width, 1120);
    }

    #[test]
    fn test_serialization() {
        let settings = Settings {
            window_width: 1920,
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.window_width, 1920);
    }

    #[test]
    fn test_partial_deserialization() {
        // Test that missing fields use defaults
        let json = r#"{"window_width": 800}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.window_width, 800);
        assert_eq!(settings.window_height, 760); // Default
    }
}
