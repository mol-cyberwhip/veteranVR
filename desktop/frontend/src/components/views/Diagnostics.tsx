import { useRef, useEffect, useState } from 'react';
import { useApp } from '../../context/AppContext';
import { useInternalLogs, LogEntry } from '../../hooks/useInternalLogs';
import { api } from '../../services/api';
import styles from './Diagnostics.module.css';

function LogLine({ entry }: { entry: LogEntry }) {
  const ts = new Date(entry.timestamp).toISOString().slice(11, 23);
  const levelClass = styles[`log-level-${entry.level}`];
  return (
    <div className={`${styles['log-line']}${levelClass ? ` ${levelClass}` : ''}`}>
      <span className={styles['log-ts']}>{ts}</span>
      <span className={styles['log-source']}>{entry.source}</span>
      <span className={styles['log-msg']}>{entry.message}</span>
      {(entry.repeatCount || 0) > 1 && <span className={styles['log-repeat']}>x{entry.repeatCount}</span>}
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

      <div className={styles['diagnostics-grid']}>
        <div className={styles['diagnostics-card']}>
          <span className="card-expanded-label">Device</span>
          <span className={`${styles['diagnostics-value']} ${deviceConnected ? styles['value-ok'] : styles['value-danger']}`}>
            {deviceConnected ? 'Connected' : 'Disconnected'}
          </span>
          <span className={styles['diagnostics-detail']}>{deviceMsg}</span>
        </div>

        <div className={styles['diagnostics-card']}>
          <span className="card-expanded-label">Catalog</span>
          <span className={`${styles['diagnostics-value']} ${catalogSynced ? styles['value-ok'] : styles['value-warn']}`}>
            {catalogSynced ? 'Synced' : 'Pending'}
          </span>
          <span className={styles['diagnostics-detail']}>{catalogMsg}</span>
        </div>

        <div className={styles['diagnostics-card']}>
          <span className="card-expanded-label">Download Queue</span>
          <span className={`${styles['diagnostics-value']} ${activeDownload ? styles['value-accent'] : styles['value-ok']}`}>
            {queueCount} queued
          </span>
          <span className={styles['diagnostics-detail']}>
            {activeDownload ? `Downloading: ${activeDownload.game_name || activeDownload.package_name}` : 'Idle'}
          </span>
        </div>
      </div>

      <div className={styles['diagnostics-logs']}>
        <div className={styles['diagnostics-logs-header']}>
          <h3 className="diagnostics-section-title">Internal Logs</h3>
          <div className={styles['diagnostics-logs-controls']}>
            <div className={styles['log-level-filters']}>
              {['all', 'error', 'warn', 'info'].map(level => (
                <button
                  key={level}
                  type="button"
                  className={`btn-sm ${styles['log-filter-chip']}${logFilter === level ? ` ${styles['active']}` : ''}`}
                  onClick={() => setLogFilter(level)}
                >
                  {level === 'all' ? 'All' : level.charAt(0).toUpperCase() + level.slice(1)}
                </button>
              ))}
            </div>
            <label className={styles['diagnostics-autoscroll']}>
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
        <div className={styles['diagnostics-log-viewer']}>
          {entries.length === 0 && (
            <div className={styles['log-empty']}>No log entries yet</div>
          )}
          {entries.filter(e => logFilter === 'all' || e.level === logFilter).map((entry, i) => (
            <LogLine key={i} entry={entry} />
          ))}
          <div ref={logEndRef} />
        </div>
      </div>

      <div className={styles['diagnostics-troubleshoot']}>
        <h3 className="diagnostics-section-title">Troubleshooting</h3>
        <p className="diagnostics-section-hint">Use these if the app stops responding or behaves unexpectedly</p>
        <div className={styles['diagnostics-actions']}>
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
