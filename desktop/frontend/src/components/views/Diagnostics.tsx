import { useRef, useEffect, useState } from 'react';
import { useApp } from '../../context/AppContext';
import { useInternalLogs, LogEntry } from '../../hooks/useInternalLogs';
import { api } from '../../services/api';

function LogLine({ entry }: { entry: LogEntry }) {
  const ts = new Date(entry.timestamp).toISOString().slice(11, 23);
  const levelClass = `log-level-${entry.level}`;
  return (
    <div className={`log-line ${levelClass}`}>
      <span className="log-ts">{ts}</span>
      <span className="log-source">{entry.source}</span>
      <span className="log-msg">{entry.message}</span>
      {(entry.repeatCount || 0) > 1 && <span className="log-repeat">x{entry.repeatCount}</span>}
    </div>
  );
}

export default function DiagnosticsView() {
  const { deviceStatus, catalogStatus, downloadQueue } = useApp();
  const { entries, clear, copyToClipboard } = useInternalLogs();
  const logEndRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);
  const [copied, setCopied] = useState(false);
  const [logFilter, setLogFilter] = useState<string>('all');

  const deviceConnected = deviceStatus?.status === 'connected' || deviceStatus?.status === 'multiple_connected';
  const deviceMsg = deviceStatus?.status_message || 'No device';
  const catalogSynced = catalogStatus?.synced === true;
  const catalogMsg = catalogStatus?.message || (catalogSynced ? 'Synced' : 'Not synced');
  const queueCount = downloadQueue?.queue?.length || 0;
  const activeDownload = downloadQueue?.active_download;

  useEffect(() => {
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [entries.length, autoScroll]);

  const handleCopy = async () => {
    await copyToClipboard();
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleRefreshBackend = async () => {
    try {
      await api.getReadyState();
    } catch (e) {
      console.error('Refresh backend failed', e);
    }
  };

  const handleRecoverBackend = async () => {
    try {
      await api.recoverBackend();
    } catch (e) {
      console.error('Recover backend failed', e);
    }
  };

  return (
    <section className="content-view active panel" id="diagnostics-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Diagnostics</h2>
          <p className="panel-subtitle">System status and troubleshooting</p>
        </div>
      </div>

      <div className="diagnostics-grid">
        <div className="diagnostics-card">
          <span className="card-expanded-label">Device</span>
          <span className={`diagnostics-value ${deviceConnected ? 'value-ok' : 'value-danger'}`}>
            {deviceConnected ? 'Connected' : 'Disconnected'}
          </span>
          <span className="diagnostics-detail">{deviceMsg}</span>
        </div>

        <div className="diagnostics-card">
          <span className="card-expanded-label">Catalog</span>
          <span className={`diagnostics-value ${catalogSynced ? 'value-ok' : 'value-warn'}`}>
            {catalogSynced ? 'Synced' : 'Pending'}
          </span>
          <span className="diagnostics-detail">{catalogMsg}</span>
        </div>

        <div className="diagnostics-card">
          <span className="card-expanded-label">Download Queue</span>
          <span className={`diagnostics-value ${activeDownload ? 'value-accent' : 'value-ok'}`}>
            {queueCount} queued
          </span>
          <span className="diagnostics-detail">
            {activeDownload ? `Downloading: ${activeDownload.game_name || activeDownload.package_name}` : 'Idle'}
          </span>
        </div>
      </div>

      <div className="diagnostics-logs">
        <div className="diagnostics-logs-header">
          <h3 className="diagnostics-section-title">Internal Logs</h3>
          <div className="diagnostics-logs-controls">
            <div className="log-level-filters">
              {['all', 'error', 'warn', 'info'].map(level => (
                <button
                  key={level}
                  type="button"
                  className={`btn-sm log-filter-chip ${logFilter === level ? 'active' : ''}`}
                  onClick={() => setLogFilter(level)}
                >
                  {level === 'all' ? 'All' : level.charAt(0).toUpperCase() + level.slice(1)}
                </button>
              ))}
            </div>
            <label className="diagnostics-autoscroll">
              <input type="checkbox" checked={autoScroll} onChange={(e) => setAutoScroll(e.target.checked)} />
              Auto-scroll
            </label>
            <button type="button" className="btn-sm btn-ghost" onClick={handleCopy}>
              {copied ? 'Copied!' : 'Copy'}
            </button>
            <button type="button" className="btn-sm btn-ghost" onClick={clear}>
              Clear
            </button>
          </div>
        </div>
        <div className="diagnostics-log-viewer">
          {entries.length === 0 && (
            <div className="log-empty">No log entries yet</div>
          )}
          {entries.filter(e => logFilter === 'all' || e.level === logFilter).map((entry, i) => (
            <LogLine key={i} entry={entry} />
          ))}
          <div ref={logEndRef} />
        </div>
      </div>

      <div className="diagnostics-troubleshoot">
        <h3 className="diagnostics-section-title">Troubleshooting</h3>
        <p className="diagnostics-section-hint">Use these if the app stops responding or behaves unexpectedly</p>
        <div className="diagnostics-actions">
          <button id="refresh-button" type="button" className="btn-secondary" onClick={handleRefreshBackend}>
            Refresh Backend State
          </button>
          <button id="recover-button" type="button" className="btn-danger" onClick={handleRecoverBackend}>
            Recover Backend
          </button>
        </div>
      </div>
    </section>
  );
}
