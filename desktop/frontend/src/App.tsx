import { useState } from 'react';
import LibraryView from './components/views/Library';
import DownloadsView from './components/views/Downloads';
import InstalledView from './components/views/Installed';
import BackupsView from './components/views/Backups';
import DiagnosticsView from './components/views/Diagnostics';
import { useApp } from './context/AppContext';

const NAV_ITEMS = [
  { id: 'library-view',     label: 'Library',     icon: '\uD83C\uDFAE' },
  { id: 'download-view',    label: 'Downloads',   icon: '\u2B07' },
  { id: 'installed-view',   label: 'Installed',   icon: '\uD83D\uDCE6' },
  { id: 'backup-view',      label: 'Backups',     icon: '\uD83D\uDCBE' },
  { id: 'diagnostics-view', label: 'Diagnostics', icon: '\u2699' },
] as const;

function App() {
  const [activeTab, setActiveTab] = useState('library-view');
  const { deviceStatus, downloadQueue, installedApps, refreshDevice } = useApp();

  const renderView = () => {
    switch (activeTab) {
      case 'library-view': return <LibraryView />;
      case 'download-view': return <DownloadsView />;
      case 'installed-view': return <InstalledView />;
      case 'backup-view': return <BackupsView />;
      case 'diagnostics-view': return <DiagnosticsView />;
      default: return <LibraryView />;
    }
  };

  const deviceConnected = deviceStatus?.status === 'connected' || deviceStatus?.status === 'multiple_connected';
  const deviceMsg = deviceStatus?.status_message || "No device connected";
  const deviceClass = deviceConnected ? "sidebar-device-status connected" : "sidebar-device-status disconnected";

  const queueCount = downloadQueue?.queue?.length || 0;

  const getBadge = (id: string) => {
    if (id === 'download-view' && queueCount > 0) return queueCount;
    if (id === 'installed-view' && installedApps.length > 0) return installedApps.length;
    return null;
  };

  return (
    <div className="app-shell">
      <header className="titlebar">
        <div className="titlebar-brand">
          <div className="brand-mark"><img src="./assets/app-logo.svg" alt="Veteran logo" /></div>
          <div className="titlebar-copy">
            <h1>Veteran Desktop</h1>
          </div>
        </div>
      </header>

      <aside className="sidebar">
        <nav className="sidebar-nav">
          {NAV_ITEMS.map(({ id, label, icon }) => {
            const badge = getBadge(id);
            return (
              <button
                key={id}
                className={`sidebar-nav-item ${activeTab === id ? 'nav-active' : ''}`}
                type="button"
                onClick={() => setActiveTab(id)}
              >
                <span className="sidebar-nav-icon">{icon}</span>
                <span className="sidebar-nav-label">{label}</span>
                {badge != null && <span className="sidebar-badge">{badge}</span>}
              </button>
            );
          })}
        </nav>

        <div className="sidebar-device-info">
          <span className="sidebar-label">Device</span>
          <select id="sidebar-device-select" className="device-select-dropdown">
            <option value="">Auto-select device</option>
            {deviceStatus?.devices?.map((d: any) => (
                <option key={d.serial} value={d.serial}>{d.serial} ({d.state})</option>
            ))}
          </select>
          <p id="sidebar-device-status" className={deviceClass}>{deviceMsg}</p>
          <button
            id="sidebar-refresh-device-button"
            type="button"
            className="sidebar-refresh-btn"
            onClick={refreshDevice}
          >Refresh Device</button>
        </div>

        <div className="sidebar-group sidebar-wireless-group">
          <span className="sidebar-label">Wireless ADB</span>
          <input id="sidebar-wireless-endpoint" type="text" placeholder="192.168.1.20:5555" className="sidebar-wireless-input" />
          <div className="sidebar-wireless-actions">
            <button id="sidebar-wireless-connect-button" type="button" className="sidebar-wireless-btn">Connect</button>
            <button id="sidebar-wireless-disconnect-button" type="button" className="sidebar-wireless-btn">Disconnect</button>
            <button id="sidebar-wireless-reconnect-button" type="button" className="sidebar-wireless-btn sidebar-wireless-btn-full">Reconnect</button>
          </div>
          <label className="sidebar-wireless-auto">
            <input id="sidebar-wireless-auto-reconnect" type="checkbox" />
            Auto-reconnect
          </label>
        </div>
      </aside>

      <main className="workspace">
        <div id="frontend-error-banner" className="frontend-error-banner hidden"></div>
        {renderView()}
      </main>

      <footer className="statusbar" id="statusbar">
        <div className="statusbar-left">
          <span className={`statusbar-dot ${deviceConnected ? 'connected' : ''}`} id="statusbar-device-dot"></span>
          <span id="statusbar-device-text">{deviceConnected ? "Device Connected" : "No device"}</span>
        </div>
        <div className="statusbar-right">
          <span id="statusbar-queue-count">Queue: {queueCount}</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
