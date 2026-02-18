use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, specta::Type)]
pub struct Game {
    pub game_name: String,
    pub release_name: String,
    pub package_name: String,
    pub version_code: String,
    pub release_apk_path: String,
    pub version_name: String,
    pub downloads: String,
    pub size: String,
    pub last_updated: String,
    pub thumbnail_path: String,
    pub thumbnail_exists: bool,
    pub note_path: String,
    pub note_excerpt: String,
    pub note_exists: bool,
    pub popularity_rank: i32,
    pub is_new: bool,
}

static DATE_PREFIX_PATTERN: OnceLock<Regex> = OnceLock::new();
static NUMERIC_PATTERN: OnceLock<Regex> = OnceLock::new();
static SIZE_WITH_UNIT_PATTERN: OnceLock<Regex> = OnceLock::new();
static VERSION_FROM_RELEASE_PATTERN: OnceLock<Regex> = OnceLock::new();

impl Game {
    pub fn from_csv_row(fields: &[&str]) -> Option<Self> {
        if fields.len() < 4 {
            return None;
        }

        let game_name = fields.first().unwrap_or(&"").trim().to_string();
        let release_name = fields.get(1).unwrap_or(&"").trim().to_string();
        let package_name = fields.get(2).unwrap_or(&"").trim().to_string();
        let version_code = fields.get(3).unwrap_or(&"").trim().to_string();

        if Self::looks_like_modern_catalog_schema(fields) {
            let last_updated = fields.get(4).unwrap_or(&"").trim().to_string();
            let size = Self::normalize_size_field(fields.get(5).unwrap_or(&"").trim());
            let downloads = fields.get(6).unwrap_or(&"").trim().to_string();
            let version_name = Self::extract_version_name_from_release(&release_name);

            Some(Game {
                game_name,
                release_name,
                package_name,
                version_code,
                release_apk_path: String::new(),
                version_name,
                downloads,
                size,
                last_updated,
                ..Default::default()
            })
        } else {
            // Legacy schema
            // 4: apk_path, 5: version_name, 6: downloads, 7: size, 8: last_updated
            Some(Game {
                game_name,
                release_name,
                package_name,
                version_code,
                release_apk_path: fields.get(4).unwrap_or(&"").trim().to_string(),
                version_name: fields.get(5).unwrap_or(&"").trim().to_string(),
                downloads: fields.get(6).unwrap_or(&"").trim().to_string(),
                size: fields.get(7).unwrap_or(&"").trim().to_string(),
                last_updated: fields.get(8).unwrap_or(&"").trim().to_string(),
                ..Default::default()
            })
        }
    }

    fn looks_like_modern_catalog_schema(fields: &[&str]) -> bool {
        if fields.len() < 7 {
            return false;
        }
        let last_updated = fields.get(4).unwrap_or(&"").trim();
        let size_field = fields.get(5).unwrap_or(&"").trim();

        let date_pattern =
            DATE_PREFIX_PATTERN.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap());
        if !date_pattern.is_match(last_updated) {
            return false;
        }

        let numeric = NUMERIC_PATTERN.get_or_init(|| Regex::new(r"^\d+(?:\.\d+)?$").unwrap());
        let size_unit = SIZE_WITH_UNIT_PATTERN.get_or_init(|| {
            Regex::new(r"(?i)^\d+(?:\.\d+)?\s*(?:KB|MB|GB|TB|KIB|MIB|GIB|TIB)$").unwrap()
        });

        numeric.is_match(size_field) || size_unit.is_match(size_field)
    }

    fn extract_version_name_from_release(release_name: &str) -> String {
        let pattern = VERSION_FROM_RELEASE_PATTERN
            .get_or_init(|| Regex::new(r"(?i)\bv\d+\+([^\s-]+)").unwrap());
        if let Some(captures) = pattern.captures(release_name) {
            if let Some(m) = captures.get(1) {
                return m.as_str().trim().to_string();
            }
        }
        String::new()
    }

    fn normalize_size_field(raw_size: &str) -> String {
        let size_value = raw_size.trim();
        if size_value.is_empty() {
            return String::new();
        }

        let size_unit = SIZE_WITH_UNIT_PATTERN.get_or_init(|| {
            Regex::new(r"(?i)^\d+(?:\.\d+)?\s*(?:KB|MB|GB|TB|KIB|MIB|GIB|TIB)$").unwrap()
        });
        if size_unit.is_match(size_value) {
            return size_value.to_string();
        }

        let numeric = NUMERIC_PATTERN.get_or_init(|| Regex::new(r"^\d+(?:\.\d+)?$").unwrap());
        if numeric.is_match(size_value) {
            let normalized = if size_value.contains('.') {
                size_value
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            } else {
                size_value.to_string()
            };
            return format!("{} MB", normalized);
        }

        size_value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modern_schema() {
        let row = "My Game;My Release v1+123;com.game;100;2023-01-01;1024;1000";
        let fields: Vec<&str> = row.split(';').collect();
        let game = Game::from_csv_row(&fields).unwrap();

        assert_eq!(game.game_name, "My Game");
        assert_eq!(game.release_name, "My Release v1+123");
        assert_eq!(game.package_name, "com.game");
        assert_eq!(game.version_code, "100");
        assert_eq!(game.last_updated, "2023-01-01");
        assert_eq!(game.size, "1024 MB"); // Normalized
        assert_eq!(game.downloads, "1000");
        assert_eq!(game.version_name, "123"); // Extracted from release name
    }

    #[test]
    fn test_parse_legacy_schema() {
        let row = "Old Game;Old Release;com.old;50;/path/to/apk;1.0;500;500 MB;2022-01-01";
        let fields: Vec<&str> = row.split(';').collect();
        let game = Game::from_csv_row(&fields).unwrap();

        assert_eq!(game.game_name, "Old Game");
        assert_eq!(game.release_apk_path, "/path/to/apk");
        assert_eq!(game.version_name, "1.0");
        assert_eq!(game.downloads, "500");
        assert_eq!(game.size, "500 MB");
        assert_eq!(game.last_updated, "2022-01-01");
    }

    #[test]
    fn test_normalize_size() {
        assert_eq!(Game::normalize_size_field("1024"), "1024 MB");
        assert_eq!(Game::normalize_size_field("1.5"), "1.5 MB");
        assert_eq!(Game::normalize_size_field("1.0"), "1 MB");
        assert_eq!(Game::normalize_size_field("2 GB"), "2 GB");
        assert_eq!(Game::normalize_size_field(""), "");
    }

    #[test]
    fn test_extract_version_name() {
        assert_eq!(
            Game::extract_version_name_from_release("Game v1+1.2.3"),
            "1.2.3"
        );
        // Python regex `[^\s-]+` stops at dash, so "build-456" -> "build"
        assert_eq!(
            Game::extract_version_name_from_release("Game v2+build-456"),
            "build"
        );
        assert_eq!(
            Game::extract_version_name_from_release("No Version Here"),
            ""
        );
    }
}
