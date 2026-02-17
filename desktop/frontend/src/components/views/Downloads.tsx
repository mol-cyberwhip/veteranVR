import { useState } from 'react';
import { useApp } from '../../context/AppContext';
import { DownloadItem } from '../Downloads/DownloadItem';
import { api } from '../../services/api';

export default function DownloadsView() {
  const { downloadQueue, gameMap, refreshQueue } = useApp();
  const [error, setError] = useState<string | null>(null);

  const handleStartProcessing = async () => {
    try {
        await api.startDownloadProcessing();
        await refreshQueue();
    } catch (e: any) { setError(e.message); }
  };

  const handleCancel = async (pkg: string) => {
    try {
        await api.cancelDownload(pkg);
        await refreshQueue();
    } catch (e: any) { setError(e.message); }
  };

  const handleInstall = async (pkg: string) => {
    try {
        await api.installGame(pkg);
    } catch (e: any) { setError(e.message); }
  };

  const handleOpenFolder = async () => {
    try {
        await api.openDownloadFolder();
    } catch (e: any) { setError(e.message); }
  };

  const handlePause = async (pkg: string) => {
    try {
        await api.pauseItem(pkg);
        await refreshQueue();
    } catch (e: any) { setError(e.message); }
  };

  const handleResume = async (pkg: string) => {
    try {
        await api.resumeItem(pkg);
        await refreshQueue();
    } catch (e: any) { setError(e.message); }
  };

  const handleRetry = async (pkg: string) => {
    try {
        await api.queueDownload(pkg);
        await refreshQueue();
    } catch (e: any) { setError(e.message); }
  };

  const queue = downloadQueue?.queue || [];
  const activeDownload = downloadQueue?.active_download;

  // Items are active if they are in the active_download slot OR in the queue with an active status.
  // We use a Map keyed by package+release to avoid duplicates if the backend reports the same item in both.
  const activeMap = new Map();
  if (activeDownload) {
    activeMap.set(`${activeDownload.package_name}|${activeDownload.release_name}`, activeDownload);
  }
  queue.forEach((i: any) => {
    if (i.status === 'downloading' || i.status === 'extracting' || i.status === 'paused') {
      activeMap.set(`${i.package_name}|${i.release_name}`, i);
    }
  });
  const activeDownloads = Array.from(activeMap.values());
  const activePkgNames = new Set(activeMap.keys());

  const pendingDownloads = queue.filter((i: any) => 
    i.status !== 'downloading' && 
    i.status !== 'extracting' && 
    i.status !== 'completed' && 
    i.status !== 'cancelled' &&
    i.status !== 'failed' &&
    !activePkgNames.has(`${i.package_name}|${i.release_name}`)
  );
  const completedDownloads = queue.filter((i: any) => i.status === 'completed');
  const failedDownloads = queue.filter((i: any) => i.status === 'failed' || i.status === 'cancelled');

  const totalItems = activeDownloads.length + pendingDownloads.length + completedDownloads.length + failedDownloads.length;
  const subtitle = activeDownloads.length > 0
    ? `Downloading ${activeDownloads.length} item${activeDownloads.length > 1 ? 's' : ''}`
    : totalItems > 0
      ? `${totalItems} item${totalItems > 1 ? 's' : ''} in queue`
      : 'No active downloads';

  return (
    <section className="content-view active panel" id="download-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Download Queue</h2>
          <p className="panel-subtitle">{subtitle}</p>
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
          {pendingDownloads.length > 0 && (
            <button id="start-download-processing-button" type="button" className="btn-success" onClick={handleStartProcessing}>Start Processing</button>
          )}
          <button type="button" className="btn-secondary" onClick={handleOpenFolder}>Open Downloads Folder</button>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      {totalItems === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">&#128229;</div>
          <div className="empty-state-title">No downloads</div>
          <div className="empty-state-hint">Browse the Library and queue games for download</div>
        </div>
      ) : (
        <div className="download-list-container">
          {activeDownloads.length > 0 && (
            <div className="download-section">
              <h3 className="download-section-header">Active</h3>
              {activeDownloads.map((item: any) => (
                <DownloadItem
                  key={`${item.package_name}-${item.release_name}`}
                  item={item}
                  game={gameMap.get(`${item.package_name}|${item.release_name}`) || gameMap.get(item.package_name)}
                  onCancel={handleCancel}
                  onInstall={handleInstall}
                  onRetry={handleRetry}
                  onPause={handlePause}
                  onResume={handleResume}
                />
              ))}
            </div>
          )}

          {pendingDownloads.length > 0 && (
            <div className="download-section">
              <h3 className="download-section-header">Queued ({pendingDownloads.length})</h3>
              {pendingDownloads.map((item: any) => (
                <DownloadItem
                  key={`${item.package_name}-${item.release_name}`}
                  item={item}
                  game={gameMap.get(`${item.package_name}|${item.release_name}`) || gameMap.get(item.package_name)}
                  onCancel={handleCancel}
                  onInstall={handleInstall}
                  onRetry={handleRetry}
                  onPause={handlePause}
                  onResume={handleResume}
                />
              ))}
            </div>
          )}

          {failedDownloads.length > 0 && (
            <div className="download-section">
              <h3 className="download-section-header">Failed / Cancelled</h3>
              {failedDownloads.map((item: any) => (
                <DownloadItem
                  key={`${item.package_name}-${item.release_name}`}
                  item={item}
                  game={gameMap.get(`${item.package_name}|${item.release_name}`) || gameMap.get(item.package_name)}
                  onCancel={handleCancel}
                  onInstall={handleInstall}
                  onRetry={handleRetry}
                  onPause={handlePause}
                  onResume={handleResume}
                />
              ))}
            </div>
          )}

          {completedDownloads.length > 0 && (
            <div className="download-section">
              <h3 className="download-section-header">Completed</h3>
              {completedDownloads.map((item: any) => (
                <DownloadItem
                  key={`${item.package_name}-${item.release_name}`}
                  item={item}
                  game={gameMap.get(`${item.package_name}|${item.release_name}`) || gameMap.get(item.package_name)}
                  onCancel={handleCancel}
                  onInstall={handleInstall}
                  onRetry={handleRetry}
                  onPause={handlePause}
                  onResume={handleResume}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </section>
  );
}
