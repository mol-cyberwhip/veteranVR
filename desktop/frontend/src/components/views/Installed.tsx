import { useState } from 'react';
import { useApp } from '../../context/AppContext';

export default function InstalledView() {
  const { installedApps, gameMap, deviceStatus, refreshDevice, installingPackages, uninstallingPackages, startInstall, startUninstall } = useApp();
  const [filter, setFilter] = useState('');
  const [error, setError] = useState<string | null>(null);

  const deviceConnected = deviceStatus?.status === 'connected' || deviceStatus?.status === 'multiple_connected';

  const filteredApps = installedApps.filter((app: any) => {
    const pkg = app.package_name || '';
    const name = gameMap.get(pkg)?.game_name || pkg;
    return name.toLowerCase().includes(filter.toLowerCase()) || pkg.toLowerCase().includes(filter.toLowerCase());
  });

  const updateCount = installedApps.filter((app: any) => {
    const game = gameMap.get(app.package_name);
    return game && game.version_code > (app.version_code || 0);
  }).length;

  const handleUninstall = async (pkg: string) => {
    if (!window.confirm(`Are you sure you want to uninstall ${pkg}?`)) return;
    try {
      await startUninstall(pkg);
    } catch (e: any) {
      setError(`${e.message || 'Failed to uninstall'}. Check Diagnostics for details.`);
    }
  };

  const handleUpdate = async (pkg: string) => {
    try {
        await startInstall(pkg);
    } catch (e: any) {
        setError(`${e.message || 'Failed to start install'}. Check Diagnostics for details.`);
    }
  };

  return (
    <section className="content-view active panel" id="installed-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Installed Apps</h2>
          <p className="panel-subtitle">Apps installed on connected device</p>
        </div>
        <div className="actions">
            <button
                id="refresh-installed-apps-button-new"
                type="button"
                className="btn-sm btn-secondary"
                onClick={() => refreshDevice()}
            >
                Refresh
            </button>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      {!deviceConnected ? (
        <div className="empty-state">
          <div className="empty-state-icon">&#128268;</div>
          <div className="empty-state-title">No device connected</div>
          <div className="empty-state-hint">Connect your Quest via USB or Wireless ADB to see installed apps</div>
        </div>
      ) : (
        <>
          <div className="installed-summary" id="installed-summary">
            <span><strong>{installedApps.length}</strong> apps installed</span>
            {updateCount > 0 && <span><strong>{updateCount}</strong> update{updateCount > 1 ? 's' : ''} available</span>}
          </div>

          <input
            type="text"
            id="installed-filter-input"
            className="installed-filter-input"
            placeholder="Search installed apps..."
            autoComplete="off"
            spellCheck={false}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          />

          <div id="installed-apps-list-new" className="installed-list">
            {filteredApps.length === 0 && (
              <div className="empty-state">
                <div className="empty-state-title">No matching apps</div>
                <div className="empty-state-hint">Try a different search term</div>
              </div>
            )}
            {filteredApps.map((app: any) => {
                const pkg = app.package_name;
                const game = gameMap.get(pkg);
                const name = game?.game_name || pkg;
                const version = app.version_name ? `${app.version_name} (${app.version_code})` : app.version_code;
                const hasUpdate = game && game.version_code > (app.version_code || 0);

                return (
                    <div key={pkg} className="installed-item">
                        <div className="installed-thumb">
                          {(name || "?").substring(0, 2).toUpperCase()}
                        </div>
                        <div className="installed-info">
                            <div className={`installed-name ${hasUpdate ? 'has-update' : ''}`}>{name}</div>
                            <div className="installed-package">{pkg}</div>
                            <div className="installed-version">v{version}</div>
                        </div>
                        <div className="installed-actions">
                            {hasUpdate && (
                                <button
                                    className={`btn-sm install-accent${installingPackages.has(pkg) ? ' btn-installing' : ''}`}
                                    onClick={() => handleUpdate(pkg)}
                                    disabled={installingPackages.has(pkg) || uninstallingPackages.has(pkg)}
                                >
                                    {installingPackages.has(pkg) && <span className="btn-spinner" />}
                                    {installingPackages.has(pkg) ? (installingPackages.get(pkg) || 'Installing...') : 'Update'}
                                </button>
                            )}
                            <button
                                className={`btn-sm btn-danger${uninstallingPackages.has(pkg) ? ' btn-installing' : ''}`}
                                onClick={() => handleUninstall(pkg)}
                                disabled={uninstallingPackages.has(pkg) || installingPackages.has(pkg)}
                            >
                                {uninstallingPackages.has(pkg) && <span className="btn-spinner" />}
                                {uninstallingPackages.has(pkg) ? 'Uninstalling...' : 'Uninstall'}
                            </button>
                        </div>
                    </div>
                );
            })}
          </div>
        </>
      )}
    </section>
  );
}
