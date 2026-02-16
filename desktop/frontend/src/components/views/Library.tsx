import { useState, useEffect } from 'react';
import { api } from '../../services/api';
import { useApp } from '../../context/AppContext';
import { Game } from '../../types';
import { GameCard } from '../Library/GameCard';

export default function LibraryView() {
  const { catalogStatus, downloadQueue, installedApps, installingPackages, startInstall } = useApp();
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filter state
  const [search, setSearch] = useState('');
  const [sortBy, setSortBy] = useState('popularity');
  const [sortAsc, setSortAsc] = useState(true);
  const [filter, setFilter] = useState('all');

  // Map installed packages for quick lookup
  const installedMap = new Map();
  installedApps.forEach(app => installedMap.set(app.package_name, app.version_code));

  // Map download queue
  const queueItems = downloadQueue?.queue || [];
  const activeDownload = downloadQueue?.active_download;
  
  const getDownloadState = (pkg: string, releaseName?: string) => {
      if (activeDownload?.package_name === pkg && (!releaseName || activeDownload.release_name === releaseName)) {
          return { status: 'downloading', progress: activeDownload.progress_percent };
      }
      const queued = queueItems.find((i: any) => 
          i.package_name === pkg && (!releaseName || i.release_name === releaseName)
      );
      if (queued) return { status: queued.status, progress: queued.progress_percent };
      return { status: undefined, progress: 0 };
  };

  const fetchLibrary = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getLibrary(search, sortBy, sortAsc, filter, 500, 0);
      setGames(result.games);
    } catch (err: any) {
      setError(err.message || 'Failed to load library');
    } finally {
      setLoading(false);
    }
  };

  // Debounce search
  useEffect(() => {
    const timer = setTimeout(() => {
      fetchLibrary();
    }, 300);
    return () => clearTimeout(timer);
  }, [search, sortBy, sortAsc, filter, catalogStatus?.synced]);

  const handleDownload = async (pkg: string, releaseName?: string) => {
    try {
        await api.queueDownload(pkg, releaseName);
        await api.startDownloadProcessing();
    } catch (e: any) {
        setError(e.message || 'Failed to queue download');
    }
  };

    const handleInstall = async (pkg: string, releaseName?: string) => {
      try {
          await startInstall(pkg, releaseName);
      } catch (e: any) {
          setError(`${e.message || 'Failed to start install'}. Check Diagnostics for details.`);
      }
    };

  return (
    <section className="content-view active panel" id="library-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Game Library</h2>
          <p className="panel-subtitle">Browse and manage your VR game library</p>
        </div>
        <button 
            id="catalog-sync-button" 
            type="button" 
            className="library-sync-button"
            onClick={() => api.syncCatalog()}
            disabled={catalogStatus?.sync_in_progress}
        >
            {catalogStatus?.sync_in_progress ? 'Syncing...' : 'Sync Catalog'}
        </button>
      </div>

      <div className="library-toolbar">
            <input
              id="library-search-input"
              type="text"
              placeholder="Search games..."
              autoComplete="off"
              spellCheck={false}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <select 
                id="library-sort-select" 
                value={sortBy} 
                onChange={(e) => setSortBy(e.target.value)}
            >
              <option value="name">Sort: Name</option>
              <option value="popularity">Sort: Popularity</option>
              <option value="date">Sort: Date</option>
              <option value="size">Sort: Size</option>
            </select>
            <button 
                id="library-sort-dir-button" 
                type="button" 
                title="Toggle sort direction"
                onClick={() => setSortAsc(!sortAsc)}
            >
                {sortAsc ? '↑' : '↓'}
            </button>
      </div>

      <div className="library-filters">
        {['all', 'favorites', 'new', 'popular'].map(f => (
            <button 
                key={f}
                className={`filter-chip ${filter === f ? 'active' : ''}`} 
                onClick={() => setFilter(f)}
                data-filter={f}
            >
                {f.charAt(0).toUpperCase() + f.slice(1)}
            </button>
        ))}
      </div>

      <div id="library-game-container" className="library-grid-view">
        {loading && <p style={{color:'#999',textAlign:'center',padding:'24px'}}>Loading...</p>}
        {error && <p style={{color:'#f88',textAlign:'center',padding:'24px'}}>{error}</p>}
        {!loading && !error && games.length === 0 && (
            <p style={{color:'#999',textAlign:'center',padding:'24px'}}>No games found. Sync catalog first.</p>
        )}
        {!loading && games.map(game => {
            const dl = getDownloadState(game.package_name, game.release_name);
            const installedVersion = installedMap.get(game.package_name);
            const isInstalled = installedVersion !== undefined;
            const hasUpdate = isInstalled && game.version_code > (installedVersion || 0);

            return (
                <GameCard
                    key={`${game.package_name}-${game.release_name}`}
                    game={game}
                    onDownload={handleDownload}
                    onInstall={handleInstall}
                    isInstalled={isInstalled}
                    hasUpdate={hasUpdate}
                    downloadStatus={dl.status}
                    downloadProgress={dl.progress}
                    installStatus={installingPackages.get(game.package_name)}
                />
            );
        })}
      </div>
    </section>
  );
}
