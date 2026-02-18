use crate::models::game::Game;
use anyhow::{Context, Result};
use md5;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CatalogService {
    games: Vec<Game>,
    all_versions: Vec<Game>,
    cache_dir: PathBuf,
    thumbnails_dir: PathBuf,
    notes_dir: PathBuf,
    syncing: bool,
}

impl Default for CatalogService {
    fn default() -> Self {
        Self::with_cache_dir(Self::default_cache_dir())
    }
}

impl CatalogService {
    fn default_cache_dir() -> PathBuf {
        dirs::home_dir()
            .map(|path| path.join(".veteran").join("cache"))
            .unwrap_or_else(|| PathBuf::from(".veteran").join("cache"))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cache_dir(cache_dir: impl Into<PathBuf>) -> Self {
        let cache_dir = cache_dir.into();
        let thumbnails_dir = cache_dir.join("thumbnails");
        let notes_dir = cache_dir.join("notes");

        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(&thumbnails_dir);
        let _ = std::fs::create_dir_all(&notes_dir);

        Self {
            games: Vec::new(),
            all_versions: Vec::new(),
            cache_dir,
            thumbnails_dir,
            notes_dir,
            syncing: false,
        }
    }

    pub fn is_syncing(&self) -> bool {
        self.syncing
    }

    pub fn set_syncing(&mut self, syncing: bool) {
        self.syncing = syncing;
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn thumbnails_dir(&self) -> &Path {
        &self.thumbnails_dir
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    pub fn parse_game_list_file(&mut self, game_list_path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(game_list_path)
            .with_context(|| format!("failed to read {}", game_list_path.display()))?;
        self.games = self.parse_game_list_content(&content);
        Ok(self.games.len())
    }

    pub fn load_from_cache(&mut self) -> bool {
        let cached_path = self.cache_dir.join("VRP-GameList.txt");
        self.parse_game_list_file(&cached_path)
            .map(|count| count > 0)
            .unwrap_or(false)
    }

    pub fn get_cache_age(&self) -> Option<f64> {
        let cached_path = self.cache_dir.join("VRP-GameList.txt");
        if let Ok(metadata) = std::fs::metadata(&cached_path) {
            if let Ok(modified) = metadata.modified() {
                return std::time::SystemTime::now()
                    .duration_since(modified)
                    .ok()
                    .map(|d| d.as_secs_f64() / 3600.0);
            }
        }
        None
    }

    pub fn search(&self, query: &str) -> Vec<Game> {
        let query = query.trim();
        if query.is_empty() {
            return self.games.clone();
        }

        if let Some(release_query) = query.strip_prefix("release:") {
            let q = release_query.trim().to_lowercase();
            return self
                .all_versions
                .iter()
                .filter(|game| game.release_name.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }

        if let Some(pkg_query) = query.strip_prefix("pkg:") {
            let q = pkg_query.trim().to_lowercase();
            return self
                .all_versions
                .iter()
                .filter(|game| game.package_name.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }

        let query_lower = query.to_lowercase();
        self.games
            .iter()
            .filter(|game| {
                game.game_name.to_lowercase().contains(&query_lower)
                    || game.release_name.to_lowercase().contains(&query_lower)
                    || game.package_name.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    pub fn get_game_by_package(&self, package_name: &str) -> Option<&Game> {
        self.games
            .iter()
            .find(|game| game.package_name == package_name)
    }

    pub fn get_games_by_package(&self, package_name: &str) -> Vec<Game> {
        fn version_sort_key(version_code: &str) -> i64 {
            version_code.parse::<i64>().unwrap_or(0)
        }

        let mut games: Vec<Game> = self
            .all_versions
            .iter()
            .filter(|game| game.package_name == package_name)
            .cloned()
            .collect();

        games.sort_by(|left, right| {
            version_sort_key(&right.version_code).cmp(&version_sort_key(&left.version_code))
        });
        games
    }

    pub fn get_game_by_package_and_release(
        &self,
        package_name: &str,
        release_name: &str,
    ) -> Option<&Game> {
        self.all_versions
            .iter()
            .find(|game| game.package_name == package_name && game.release_name == release_name)
    }

    pub fn game_name_to_hash(release_name: &str) -> String {
        // Python reference: hashlib.md5((release_name + "\n").encode("utf-8")).hexdigest()
        let input = format!("{}\n", release_name);
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    pub fn parse_game_list_content(&mut self, content: &str) -> Vec<Game> {
        let mut games_by_key: HashMap<(String, String), Game> = HashMap::new();
        let mut game_key_order: Vec<(String, String)> = Vec::new();
        let mut all_versions: Vec<Game> = Vec::new();

        // First pass: collect all games and track popularity scores per package
        let mut popularity_scores: HashMap<String, f64> = HashMap::new();

        for (i, line) in content.lines().enumerate() {
            if i == 0 {
                continue;
            } // Skip header
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(';').collect();
            if let Some(game) = Game::from_csv_row(&fields) {
                all_versions.push(game.clone());
                // Track highest popularity score per package
                if let Ok(score) = game.downloads.parse::<f64>() {
                    let package = &game.package_name;
                    if let Some(existing) = popularity_scores.get(package) {
                        if score > *existing {
                            popularity_scores.insert(package.clone(), score);
                        }
                    } else {
                        popularity_scores.insert(package.clone(), score);
                    }
                }

                let key = (game.package_name.clone(), game.game_name.clone());

                // Compare versions
                if let Some(existing) = games_by_key.get(&key) {
                    let existing_ver = existing.version_code.parse::<i64>().unwrap_or(0);
                    let new_ver = game.version_code.parse::<i64>().unwrap_or(0);

                    if new_ver > existing_ver {
                        games_by_key.insert(key, game);
                    } else if new_ver == existing_ver {
                        // Fallback string compare if numbers match
                        if game.version_code > existing.version_code {
                            games_by_key.insert(key, game);
                        }
                    }
                } else {
                    game_key_order.push(key.clone());
                    games_by_key.insert(key, game);
                }
            }
        }

        // Calculate popularity rankings
        let mut packages_with_scores: Vec<(String, f64)> = popularity_scores
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .collect();
        packages_with_scores
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let rankings: HashMap<String, i32> = packages_with_scores
            .into_iter()
            .enumerate()
            .map(|(idx, (pkg, _))| (pkg, (idx + 1) as i32))
            .collect();

        // Apply rankings to games
        let mut games: Vec<Game> = game_key_order
            .into_iter()
            .filter_map(|key| games_by_key.remove(&key))
            .collect();

        for game in &mut games {
            if let Some(rank) = rankings.get(&game.package_name) {
                game.popularity_rank = *rank;
            }
        }

        self.all_versions = all_versions;
        games
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_game_name_to_hash() {
        let hash = CatalogService::game_name_to_hash("Test Game");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_parse_deduplication() {
        let content = "Header\nGame;Rel1;com.test;10;2023-01-01;100;0\nGame;Rel2;com.test;11;2023-01-02;100;0";
        let mut service = CatalogService::new();
        let games = service.parse_game_list_content(content);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].version_code, "11");
    }

    #[test]
    fn test_load_from_cache() {
        let temp_dir = tempdir().unwrap();
        let cache_path = temp_dir.path();
        let cache_file = cache_path.join("VRP-GameList.txt");
        std::fs::write(cache_file, "Header\nA;Rel;pkg.a;1;2023-01-01;10;1").unwrap();

        let mut service = CatalogService::with_cache_dir(cache_path.to_path_buf());
        assert!(service.load_from_cache());
        assert_eq!(service.games().len(), 1);
        assert_eq!(service.games()[0].package_name, "pkg.a");
    }
}
