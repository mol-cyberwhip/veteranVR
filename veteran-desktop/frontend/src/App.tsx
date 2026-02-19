import { useState } from 'react';
import LibraryView from './components/views/Library';
import DownloadsView from './components/views/Downloads';
import InstalledView from './components/views/Installed';
import BackupsView from './components/views/Backups';
import DiagnosticsView from './components/views/Diagnostics';
import WirelessView from './components/views/Wireless';
import { useApp } from './context/AppContext';
import styles from './App.module.css';

const NAV_ITEMS = [
  { id: 'library-view',     label: 'Library',     icon: '\uD83C\uDFAE' },
  { id: 'download-view',    label: 'Downloads',   icon: '\u2B07' },
  { id: 'installed-view',   label: 'Installed',   icon: '\uD83D\uDCE6' },
  { id: 'backup-view',      label: 'Backups',     icon: '\uD83D\uDCBE' },
  { id: 'wireless-view',    label: 'Wireless',    icon: '\uD83D\uDCF6' },
  { id: 'diagnostics-view', label: 'Diagnostics', icon: '\u2699' },
] as const;

function App() {
  const [activeTab, setActiveTab] = useState('library-view');
  const { deviceStatus, downloadQueue, installedApps, refreshDevice, selectDevice } = useApp();

  const renderView = () => {
    switch (activeTab) {
      case 'library-view': return <LibraryView />;
      case 'download-view': return <DownloadsView />;
      case 'installed-view': return <InstalledView />;
      case 'backup-view': return <BackupsView />;
      case 'wireless-view': return <WirelessView />;
      case 'diagnostics-view': return <DiagnosticsView />;
      default: return <LibraryView />;
    }
  };

  const deviceConnected = deviceStatus?.status === 'connected' || deviceStatus?.status === 'multiple_connected';
  const deviceMsg = deviceStatus?.status_message || "No device connected";
  const deviceClass = deviceConnected ? `${styles['sidebar-device-status']} ${styles['connected']}` : `${styles['sidebar-device-status']} ${styles['disconnected']}`;

  const queueCount = downloadQueue?.queue?.length || 0;
  const activeDownload = downloadQueue?.active_download;

  const getBadge = (id: string) => {
    if (id === 'download-view' && queueCount > 0) return queueCount;
    if (id === 'installed-view' && installedApps.length > 0) return installedApps.length;
    return null;
  };

  return (
    <div className={styles['app-shell']}>
      <header className={styles['titlebar']}>
        <div className={styles['titlebar-brand']}>
          <div className={styles['brand-mark']}><img src="./assets/app-logo.svg" alt="Veteran logo" /></div>
          <div className={styles['titlebar-copy']}>
            <h1>Veteran Desktop</h1>
          </div>
        </div>
      </header>

      <aside className={styles['sidebar']}>
        <nav className={styles['sidebar-nav']}>
          {NAV_ITEMS.map(({ id, label, icon }) => {
            const badge = getBadge(id);
            return (
              <button
                key={id}
                className={`${styles['sidebar-nav-item']}${activeTab === id ? ` ${styles['nav-active']}` : ''}`}
                type="button"
                onClick={() => setActiveTab(id)}
              >
                <span className={styles['sidebar-nav-icon']}>{icon}</span>
                <span className={styles['sidebar-nav-label']}>{label}</span>
                {badge != null && <span className={styles['sidebar-badge']}>{badge}</span>}
              </button>
            );
          })}
        </nav>

        <div className={styles['sidebar-device-info']}>
          <span className={styles['sidebar-label']}>Device</span>
          <select
            id="sidebar-device-select"
            className={styles['device-select-dropdown']}
            value={deviceStatus?.selected_serial || ''}
            onChange={(e) => {
              if (e.target.value) selectDevice(e.target.value);
            }}
          >
            <option value="">Auto-select device</option>
            {deviceStatus?.devices?.map((d: any) => (
                <option key={d.serial} value={d.serial}>{d.serial} ({d.state})</option>
            ))}
          </select>
          <p id="sidebar-device-status" className={deviceClass}>{deviceMsg}</p>
          <button
            id="sidebar-refresh-device-button"
            type="button"
            className={`${styles['sidebar-refresh-btn']} btn-primary`}
            onClick={refreshDevice}
          >Refresh Device</button>
        </div>

      </aside>

      <main className={styles['workspace']}>
        <div id="frontend-error-banner" className={`${styles['frontend-error-banner']} ${styles['hidden']}`}></div>
        {renderView()}
      </main>

      <footer className={styles['statusbar']} id="statusbar">
        <div className={styles['statusbar-left']}>
          <span className={`${styles['statusbar-dot']}${deviceConnected ? ` ${styles['connected']}` : ''}`} id="statusbar-device-dot"></span>
          <span id="statusbar-device-text">
            {deviceConnected ? (deviceStatus?.status_message || "Device Connected") : "No device"}
          </span>
        </div>

        {activeDownload && (
          <div className={styles['statusbar-center']}>
            <span className={styles['statusbar-download-label']}>
              {activeDownload.game_name || activeDownload.package_name}
            </span>
            <div className={styles['statusbar-download-progress']}>
              <div
                className={styles['statusbar-download-progress-fill']}
                style={{ width: `${activeDownload.progress_percent || 0}%` }}
              />
            </div>
            <span className={styles['statusbar-download-pct']}>
              {(activeDownload.progress_percent || 0).toFixed(0)}%
              {activeDownload.speed ? ` (${activeDownload.speed})` : ''}
            </span>
          </div>
        )}

        <div className={styles['statusbar-right']}>
          <button
            type="button"
            className={styles['statusbar-queue-btn']}
            onClick={() => setActiveTab('download-view')}
          >
            Queue: {queueCount}
          </button>
        </div>
      </footer>
    </div>
  );
}

export default App;
