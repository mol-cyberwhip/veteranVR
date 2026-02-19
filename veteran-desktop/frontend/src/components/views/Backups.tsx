import { useState, useEffect } from 'react';
import { useApp } from '../../context/AppContext';
import { api } from '../../services/api';
import styles from './Backups.module.css';

export default function BackupsView() {
  const { installedApps, gameMap } = useApp();
  const [selectedApp, setSelectedApp] = useState('');
  const [backups, setBackups] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchBackups();
  }, []);

  const fetchBackups = async () => {
    try {
      const list = await api.getBackups();
      setBackups(Array.isArray(list) ? list : []);
    } catch (e: any) { setError(e.message); }
  };

  const handleBackup = async () => {
    if (!selectedApp) return;
    setLoading(true);
    try {
        await api.backupApp(selectedApp);
        await fetchBackups();
    } catch (e: any) { setError(e.message); }
    finally { setLoading(false); }
  };

  const handleRestore = async (pkg: string) => {
    if (!window.confirm(`Restore backup for ${pkg}? This will overwrite current save data.`)) return;
    setLoading(true);
    try {
        await api.restoreApp(pkg);
    } catch (e: any) { setError(e.message); }
    finally { setLoading(false); }
  };

  return (
    <section className="content-view active panel" id="backup-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Backups</h2>
          <p className="panel-subtitle">Backup and restore save data</p>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className={styles['backup-create-card']}>
        <h3 className={styles['backup-card-title']}>Create Backup</h3>
        <p className={styles['backup-card-subtitle']}>Select an installed app to back up its save data</p>
        <div className={styles['backup-create-row']}>
          <select
            value={selectedApp}
            onChange={(e) => setSelectedApp(e.target.value)}
            className={styles['backup-app-select']}
          >
            <option value="">-- Select Installed App --</option>
            {installedApps.map((app: any) => {
              const name = gameMap.get(app.package_name)?.game_name || app.package_name;
              return (
                <option key={app.package_name} value={app.package_name}>
                  {name}
                </option>
              );
            })}
          </select>
          <button
            type="button"
            className="btn-success"
            onClick={handleBackup}
            disabled={!selectedApp || loading}
          >
            {loading ? 'Processing...' : 'Backup'}
          </button>
        </div>
      </div>

      <div className={styles['backup-list-section']}>
        <div className={styles['backup-list-header']}>
          <h3 className={styles['backup-card-title']}>Saved Backups</h3>
          <button type="button" className="btn-sm btn-icon btn-ghost" onClick={fetchBackups} title="Refresh list">&#8635;</button>
        </div>

        {backups.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">&#128190;</div>
            <div className="empty-state-title">No backups yet</div>
            <div className="empty-state-hint">Create a backup from an installed app above</div>
          </div>
        ) : (
          <div className={styles['backup-list']}>
            {backups.map((b: any) => {
              const pkg = b.package_name;
              const name = gameMap.get(pkg)?.game_name || pkg;
              return (
                <div key={`${pkg}-${b.timestamp}`} className={styles['backup-item']}>
                  <div className={styles['backup-item-info']}>
                    <div className={styles['backup-item-name']}>{name}</div>
                    <div className={styles['backup-item-package']}>{pkg}</div>
                  </div>
                  <div className={styles['backup-item-date']}>
                    {new Date(b.timestamp * 1000).toLocaleString()}
                  </div>
                  <button
                    type="button"
                    className="btn-sm btn-secondary"
                    onClick={() => handleRestore(pkg)}
                    disabled={loading}
                  >
                    Restore
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </section>
  );
}
