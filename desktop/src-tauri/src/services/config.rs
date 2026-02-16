use crate::models::config::PublicConfig;
use anyhow::Result;
use std::path::PathBuf;

pub const CONFIG_URLS: &[&str] = &[
    "https://raw.githubusercontent.com/vrpyou/quest/main/vrp-public.json",
    "https://vrpirates.wiki/downloads/vrp-public.json",
];

#[derive(Clone)]
pub struct ConfigService {
    cache_dir: PathBuf,
}

impl ConfigService {
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.join(".veteran").join("cache"))
                .unwrap_or_else(|| PathBuf::from(".veteran").join("cache"))
        });

        let _ = std::fs::create_dir_all(&cache_dir);

        Self { cache_dir }
    }

    pub fn cache_path(&self) -> PathBuf {
        self.cache_dir.join("vrp-public.json")
    }

    pub async fn fetch_config(&self) -> Result<PublicConfig> {
        let mut last_error = None;

        for url in CONFIG_URLS {
            match self.fetch_url(url).await {
                Ok(config) => {
                    // Cache it
                    let _ = self.save_to_cache(&config);
                    return Ok(config);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // Fallback to cache
        if let Ok(config) = self.load_from_cache() {
            return Ok(config);
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No config available")))
    }

    async fn fetch_url(&self, url: &str) -> Result<PublicConfig> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let resp = client.get(url).send().await?;
        let json: serde_json::Value = resp.json().await?;
        PublicConfig::from_json(&json).map_err(|e| anyhow::anyhow!(e))
    }

    fn save_to_cache(&self, config: &PublicConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        std::fs::write(self.cache_path(), json)?;
        Ok(())
    }

    pub fn load_from_cache(&self) -> Result<PublicConfig> {
        let content = std::fs::read_to_string(self.cache_path())?;
        let config: PublicConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
