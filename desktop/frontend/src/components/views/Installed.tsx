import { useState, useEffect } from 'react';
import { useApp } from '../../context/AppContext';
import { api } from '../../services/api';

const InstalledItemThumbnail = ({ packageName, name }: { packageName: string, name: string }) => {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    api.getThumbnailPath(packageName).then(({ exists, path }) => {
      if (active && exists && path) {
        setSrc(api.convertFileSrc(path));
      }
    });
    return () => { active = false; };
  }, [packageName]);

  if (src) return <img src={src} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />;
  return <>{(name || "?").substring(0, 2).toUpperCase()}</>;
};

export default function InstalledView() {
  const { installedApps, deviceStatus, refreshDevice, installingPackages, uninstallingPackages, startInstall, startUninstall } = useApp();
  const [filter, setFilter] = useState('');
  const [error, setError] = useState<string | null>(null);

  const deviceConnected = deviceStatus?.status === 'connected' || deviceStatus?.status === 'multiple_connected';

  const filteredApps = (installedApps || []).filter((app: any) => {
    const pkg = app.package_name || '';
    const name = app.app_name || pkg;
    return name.toLowerCase().includes(filter.toLowerCase()) || pkg.toLowerCase().includes(filter.toLowerCase());
  });

  const updates = filteredApps.filter((app: any) => app.update_available);
  const inCatalog = filteredApps.filter((app: any) => app.in_catalog && !app.update_available);
  const others = filteredApps.filter((app: any) => !app.in_catalog);

  const handleUninstall = async (pkg: string) => {
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

  const renderAppList = (apps: any[], title: string) => {
    if (apps.length === 0) return null;
    return (
      <div className="installed-section">
        <div className="installed-section-title">{title} ({apps.length})</div>
        {apps.map((app: any) => {
            const pkg = app.package_name;
            const name = app.app_name || pkg;
            const version = app.version_name ? `${app.version_name} (${app.version_code})` : app.version_code;
            const size = app.size && app.size !== "0" ? app.size : null;

            return (
                <div key={pkg} className="installed-item">
                    <div className="installed-thumb">
                      <InstalledItemThumbnail packageName={pkg} name={name} />
                    </div>
                    <div className="installed-info">
                        <div className={`installed-name ${app.update_available ? 'has-update' : ''}`}>{name}</div>
                        <div className="installed-package">{pkg}</div>
                        <div className="installed-version">
                          v{version}
                          {size && <span className="installed-size-tag"> | {size}</span>}
                        </div>
                    </div>
                    <div className="installed-actions">
                        {app.update_available && (
                            <button
                                className={`btn-sm install-accent${installingPackages.has(pkg) ? ' btn-installing' : ''}`}
                                onClick={() => handleUpdate(pkg)}
                                disabled={installingPackages.has(pkg) || uninstallingPackages.has(pkg)}
                            >
                                {installingPackages.has(pkg) && <span className="btn-spinner" />}
                                {installingPackages.get(pkg) || 'Update'}
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
    );
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
            {updates.length > 0 && <span><strong>{updates.length}</strong> update{updates.length > 1 ? 's' : ''} available</span>}
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
            
            {renderAppList(updates, "Update Available")}
            {renderAppList(inCatalog, "In Catalog")}
            {renderAppList(others, "Other Installed Apps")}
          </div>
        </>
      )}
    </section>
  );
}
