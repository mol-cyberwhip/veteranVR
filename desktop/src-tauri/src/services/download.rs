use crate::models::game::Game;
use crate::services::catalog::CatalogService;
use crate::services::rclone::{DownloadProgress, RcloneService};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadItem {
    pub operation_id: String,
    pub game: Game,
    pub status: DownloadStatus,
    pub progress: DownloadProgress,
    pub error: String,
}

impl DownloadItem {
    pub fn new(game: Game) -> Self {
        Self {
            operation_id: Uuid::new_v4().to_string(),
            game,
            status: DownloadStatus::Queued,
            progress: DownloadProgress::default(),
            error: String::new(),
        }
    }

    pub fn game_hash(&self) -> String {
        CatalogService::game_name_to_hash(&self.game.release_name)
    }
}

// Internal structure to track active downloads
type ProgressUpdate = (String, DownloadProgress);

#[derive(Debug)]
pub struct DownloadService {
    rclone: Arc<RcloneService>,
    download_dir: PathBuf,
    bandwidth_limit_mbps: f64,
    queue: Arc<RwLock<Vec<DownloadItem>>>,
    processing: Arc<RwLock<bool>>,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
    progress_rx: Arc<Mutex<mpsc::UnboundedReceiver<ProgressUpdate>>>,
    active_handles: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl Clone for DownloadService {
    fn clone(&self) -> Self {
        Self {
            rclone: self.rclone.clone(),
            download_dir: self.download_dir.clone(),
            bandwidth_limit_mbps: self.bandwidth_limit_mbps,
            queue: self.queue.clone(),
            processing: self.processing.clone(),
            progress_tx: self.progress_tx.clone(),
            progress_rx: self.progress_rx.clone(),
            active_handles: self.active_handles.clone(),
        }
    }
}

impl DownloadService {
    pub fn new(rclone: RcloneService, download_dir: PathBuf, bandwidth_limit_mbps: f64) -> Self {
        // Ensure download directory exists
        if !download_dir.exists() {
            let _ = std::fs::create_dir_all(&download_dir);
        }
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        Self {
            rclone: Arc::new(rclone),
            download_dir,
            bandwidth_limit_mbps,
            queue: Arc::new(RwLock::new(Vec::new())),
            processing: Arc::new(RwLock::new(false)),
            progress_tx,
            progress_rx: Arc::new(Mutex::new(progress_rx)),
            active_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_with_arc(rclone: Arc<RcloneService>, download_dir: PathBuf, bandwidth_limit_mbps: f64) -> Self {
        // Ensure download directory exists
        if !download_dir.exists() {
            let _ = std::fs::create_dir_all(&download_dir);
        }
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        Self {
            rclone,
            download_dir,
            bandwidth_limit_mbps,
            queue: Arc::new(RwLock::new(Vec::new())),
            processing: Arc::new(RwLock::new(false)),
            progress_tx,
            progress_rx: Arc::new(Mutex::new(progress_rx)),
            active_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn queue(&self) -> Vec<DownloadItem> {
        self.queue.read().await.clone()
    }

    pub async fn is_processing(&self) -> bool {
        *self.processing.read().await
    }

    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    pub async fn add_to_queue(&self, game: Game) -> bool {
        let mut queue = self.queue.write().await;
        if queue.iter().any(|item| item.game.package_name == game.package_name) {
            return false;
        }
        queue.push(DownloadItem::new(game));
        true
    }

    pub async fn remove_from_queue(&self, package_name: &str) -> bool {
        let mut queue = self.queue.write().await;
        let before = queue.len();
        queue.retain(|item| {
            item.game.package_name != package_name || item.status == DownloadStatus::Downloading
        });
        queue.len() != before
    }

    pub async fn reorder_queue(&self, package_name: &str, new_position: usize) -> bool {
        let mut queue = self.queue.write().await;
        let Some(index) = queue
            .iter()
            .position(|item| item.game.package_name == package_name)
        else {
            return false;
        };

        let item = queue.remove(index);
        let target = new_position.min(queue.len());
        queue.insert(target, item);
        true
    }

    pub async fn cancel_current(&self) -> Result<bool> {
        let mut queue = self.queue.write().await;
        if let Some(item) = queue
            .iter_mut()
            .find(|item| item.status == DownloadStatus::Downloading)
        {
            item.status = DownloadStatus::Cancelled;
            drop(queue); // Release lock before await
            self.rclone.cancel_download().await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn process_queue(&self) -> Result<()> {
        // Check if already processing
        {
            let processing = self.processing.read().await;
            if *processing {
                return Ok(());
            }
        }

        // Mark as processing
        {
            let mut processing = self.processing.write().await;
            *processing = true;
        }

        // Spawn the queue processor as a background task
        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.run_queue_processor().await;
        });

        // Spawn progress updater
        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.run_progress_updater().await;
        });

        Ok(())
    }

    async fn run_progress_updater(&self) {
        let mut rx = self.progress_rx.lock().await;
        while let Some((package_name, progress)) = rx.recv().await {
            let mut queue = self.queue.write().await;
            if let Some(item) = queue.iter_mut().find(|i| i.game.package_name == package_name) {
                item.progress = progress;
            }
        }
    }

    async fn run_queue_processor(&self) {
        loop {
            // Find next queued item
            let next_item = {
                let queue = self.queue.read().await;
                queue
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.status == DownloadStatus::Queued)
                    .map(|(idx, item)| (idx, item.clone()))
            };

            if let Some((index, item)) = next_item {
                // Mark as downloading
                {
                    let mut queue = self.queue.write().await;
                    queue[index].status = DownloadStatus::Downloading;
                }

                // Spawn the download as a background task
                let package_name = item.game.package_name.clone();
                let handle = self.spawn_download_task(item).await;
                
                {
                    let mut handles = self.active_handles.lock().await;
                    handles.insert(package_name, handle);
                }
            } else {
                // No more queued items
                break;
            }
        }

        // Mark as not processing
        {
            let mut processing = self.processing.write().await;
            *processing = false;
        }
    }

    async fn spawn_download_task(&self, item: DownloadItem) -> tokio::task::JoinHandle<()> {
        let rclone = self.rclone.clone();
        let download_dir = self.download_dir.clone();
        let bandwidth_limit = self.bandwidth_limit_mbps;
        let queue = self.queue.clone();
        let progress_tx = self.progress_tx.clone();
        let package_name = item.game.package_name.clone();

        tokio::spawn(async move {
            let game_hash = item.game_hash();
            let game_dir = download_dir.join(&game_hash);

            // Create progress callback that sends to our channel
            let (rclone_tx, mut rclone_rx) = mpsc::unbounded_channel::<DownloadProgress>();
            let progress_tx_clone = progress_tx.clone();
            let package_name_clone = package_name.clone();
            
            let progress_forwarder = tokio::spawn(async move {
                while let Some(progress) = rclone_rx.recv().await {
                    let _ = progress_tx_clone.send((package_name_clone.clone(), progress));
                }
            });

            // Run the download
            let result = rclone
                .download_game(&game_hash, &game_dir, bandwidth_limit, Some(rclone_tx))
                .await;

            // Wait for progress forwarder to finish
            let _ = progress_forwarder.await;

            // Update final status
            let mut queue = queue.write().await;
            if let Some(item) = queue.iter_mut().find(|i| i.game.package_name == package_name) {
                if item.status == DownloadStatus::Cancelled {
                    return;
                }

                match result {
                    Ok(download_result) if download_result.success() => {
                        item.status = DownloadStatus::Completed;
                        item.progress.percent = 100.0;
                    }
                    Ok(download_result) => {
                        item.status = DownloadStatus::Failed;
                        item.error = download_result.stderr;
                    }
                    Err(error) => {
                        item.status = DownloadStatus::Failed;
                        item.error = error.to_string();
                    }
                }
            }
        })
    }

    pub async fn process_queue_with_callback<F, Fut>(&self, on_update: F) -> Result<()>
    where
        F: Fn(DownloadItem) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        // Check if already processing
        {
            let processing = self.processing.read().await;
            if *processing {
                return Ok(());
            }
        }

        // Mark as processing
        {
            let mut processing = self.processing.write().await;
            *processing = true;
        }

        let on_update = Arc::new(on_update);
        let queue = self.queue.clone();
        let rclone = self.rclone.clone();
        let download_dir = self.download_dir.clone();
        let bandwidth_limit = self.bandwidth_limit_mbps;
        let processing = self.processing.clone();

        // Spawn queue processor with callback
        tokio::spawn(async move {
            let mut idx = 0usize;
            loop {
                // Find next queued item
                let next_item = {
                    let queue = queue.read().await;
                    queue
                        .iter()
                        .enumerate()
                        .skip(idx)
                        .find(|(_, item)| item.status == DownloadStatus::Queued)
                        .map(|(i, item)| (i, item.clone()))
                };

                if let Some((index, mut item)) = next_item {
                    idx = index + 1;
                    
                    // Mark as downloading
                    item.status = DownloadStatus::Downloading;
                    {
                        let mut queue = queue.write().await;
                        if let Some(qitem) = queue.iter_mut().find(|i| i.game.package_name == item.game.package_name) {
                            qitem.status = DownloadStatus::Downloading;
                        }
                    }
                    on_update(item.clone()).await;

                    // Run download
                    let game_hash = item.game_hash();
                    let game_dir = download_dir.join(&game_hash);
                    let (tx, mut rx) = mpsc::unbounded_channel();

                    let rclone_clone = rclone.clone();
                    let bandwidth = bandwidth_limit;
                    
                    let download_task = async move {
                        rclone_clone
                            .download_game(&game_hash, &game_dir, bandwidth, Some(tx))
                            .await
                    };

                    tokio::pin!(download_task);

                    let result = loop {
                        tokio::select! {
                            res = &mut download_task => break res,
                            Some(progress) = rx.recv() => {
                                let mut queue = queue.write().await;
                                if let Some(qitem) = queue.iter_mut().find(|i| i.game.package_name == item.game.package_name) {
                                    qitem.progress = progress;
                                }
                                drop(queue);
                                on_update(item.clone()).await;
                            }
                        }
                    };

                    // Update final status
                    {
                        let mut queue = queue.write().await;
                        if let Some(qitem) = queue.iter_mut().find(|i| i.game.package_name == item.game.package_name) {
                            if qitem.status == DownloadStatus::Cancelled {
                                on_update(qitem.clone()).await;
                                continue;
                            }

                            match result {
                                Ok(download_result) if download_result.success() => {
                                    qitem.status = DownloadStatus::Completed;
                                    qitem.progress.percent = 100.0;
                                }
                                Ok(download_result) => {
                                    qitem.status = DownloadStatus::Failed;
                                    qitem.error = download_result.stderr;
                                }
                                Err(error) => {
                                    qitem.status = DownloadStatus::Failed;
                                    qitem.error = error.to_string();
                                }
                            }
                            on_update(qitem.clone()).await;
                        }
                    }
                } else {
                    break;
                }
            }

            // Mark as not processing
            let mut processing = processing.write().await;
            *processing = false;
        });

        Ok(())
    }

    pub fn get_download_dir(&self, game: &Game) -> PathBuf {
        let hash = CatalogService::game_name_to_hash(&game.release_name);
        self.download_dir.join(hash)
    }

    pub async fn is_downloaded(&self, game: &Game) -> bool {
        let game_dir = self.get_download_dir(game);
        if !game_dir.exists() {
            return false;
        }

        has_apk(&game_dir) || game_dir.join("install.txt").exists()
    }
}

fn has_apk(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    for entry in entries.flatten() {
        if entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("apk"))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_game(package_name: &str) -> Game {
        Game {
            game_name: "Sample".to_string(),
            release_name: format!("Release {package_name}"),
            package_name: package_name.to_string(),
            version_code: "1".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn add_to_queue_rejects_duplicates() {
        let temp = tempdir().unwrap();
        let rclone = RcloneService::new(Some("rclone".to_string()));
        let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

        assert!(service.add_to_queue(sample_game("com.test")).await);
        assert!(!service.add_to_queue(sample_game("com.test")).await);
        assert_eq!(service.queue().await.len(), 1);
    }

    #[tokio::test]
    async fn remove_from_queue_removes_non_downloading_item() {
        let temp = tempdir().unwrap();
        let rclone = RcloneService::new(Some("rclone".to_string()));
        let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

        service.add_to_queue(sample_game("com.one")).await;
        service.add_to_queue(sample_game("com.two")).await;

        assert!(service.remove_from_queue("com.one").await);
        assert_eq!(service.queue().await.len(), 1);
        assert_eq!(service.queue().await[0].game.package_name, "com.two");
    }

    #[tokio::test]
    async fn reorder_queue_changes_item_position() {
        let temp = tempdir().unwrap();
        let rclone = RcloneService::new(Some("rclone".to_string()));
        let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

        service.add_to_queue(sample_game("com.one")).await;
        service.add_to_queue(sample_game("com.two")).await;
        service.add_to_queue(sample_game("com.three")).await;

        assert!(service.reorder_queue("com.three", 0).await);

        let order = service
            .queue()
            .await
            .iter()
            .map(|item| item.game.package_name.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["com.three", "com.one", "com.two"]);
    }

    #[tokio::test]
    async fn is_downloaded_detects_apk_or_install_txt() {
        let temp = tempdir().unwrap();
        let rclone = RcloneService::new(Some("rclone".to_string()));
        let service = DownloadService::new(rclone, temp.path().to_path_buf(), 0.0);

        let game = sample_game("com.downloaded");
        let dir = service.get_download_dir(&game);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!service.is_downloaded(&game).await);

        std::fs::write(dir.join("install.txt"), "adb shell echo hi").unwrap();
        assert!(service.is_downloaded(&game).await);
    }
}
