import { useState, useEffect } from 'react';
import { Game } from '../../types';
import { api } from '../../services/api';

interface GameCardProps {
  game: Game;
  onDownload: (pkg: string, releaseName?: string) => void;
  onInstall: (pkg: string, releaseName?: string) => void;
  isInstalled: boolean;
  hasUpdate: boolean;
  downloadStatus?: string; // "queued", "downloading", "completed"
  downloadProgress?: number;
  installStatus?: string; // e.g. "Installing...", "Extracting archives..."
}

export const GameCard: React.FC<GameCardProps> = ({ game, onDownload, onInstall, isInstalled, hasUpdate, downloadStatus, downloadProgress, installStatus }) => {
  const [expanded, setExpanded] = useState(false);
  const [favorite, setFavorite] = useState(game.is_favorite);
  const [thumbnailSrc, setThumbnailSrc] = useState<string | null>(null);
  const [selectedRelease] = useState<string | undefined>(game.release_name);

  useEffect(() => {
    let active = true;
    const loadThumbnail = async () => {
        try {
            const { exists, path } = await api.getThumbnailPath(game.package_name);
            if (active && exists && path) {
                setThumbnailSrc(api.convertFileSrc(path));
            }
        } catch (e) {
            console.error("Thumbnail load failed", e);
        }
    };
    loadThumbnail();
    return () => { active = false; };
  }, [game.package_name]);

  const toggleFavorite = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setFavorite(!favorite);
  };

  const handleCardClick = () => {
    setExpanded(!expanded);
  };

  const handleAction = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isInstalled && !hasUpdate) {
        onInstall(game.package_name, selectedRelease);
    } else if (isInstalled && hasUpdate) {
        onInstall(game.package_name, selectedRelease);
    } else if (downloadStatus === 'completed') {
        onInstall(game.package_name, selectedRelease);
    } else {
        onDownload(game.package_name, selectedRelease);
    }
  };

  const isInstalling = !!installStatus;

  const getActionLabel = () => {
    if (isInstalling) return installStatus!;
    if (downloadStatus === 'downloading') return `Downloading ${downloadProgress?.toFixed(0)}%`;
    if (downloadStatus === 'queued') return 'Pending...';
    if (isInstalled && hasUpdate) return 'Update';
    if (isInstalled) return 'Reinstall';
    if (downloadStatus === 'completed') return 'Install';
    return 'Download & Install';
  };

  const isBusy = isInstalling || downloadStatus === 'downloading' || downloadStatus === 'queued';
  const actionClass = 'install-accent';

  const badges = [];
  if (game.is_new) badges.push(<span key="new" className="badge badge-new">New</span>);
  if (game.popularity_rank <= 3 && game.popularity_rank > 0) badges.push(<span key="pop" className="badge badge-popular">Popular</span>);
  if (isInstalled) badges.push(<span key="inst" className="badge badge-installed">Installed</span>);
  if (hasUpdate) badges.push(<span key="upd" className="badge badge-update">Update</span>);

  const fallbackLabel = (game.game_name || "?").substring(0, 2).toUpperCase();

  return (
    <div className={`game-card ${expanded ? 'expanded' : ''}`} onClick={handleCardClick} data-package={game.package_name}>
      <div className="card-thumb">
        {thumbnailSrc ? (
            <img src={thumbnailSrc} alt="" />
        ) : (
            <span data-thumb-fallback>{fallbackLabel}</span>
        )}
        <button className={`card-fav ${favorite ? 'active' : ''}`} onClick={toggleFavorite}>
            {favorite ? '\u2665' : '\u2661'}
        </button>
        <div className="card-badges">{badges}</div>
        <div className="card-hover-overlay">
          <button
            type="button"
            className={`card-hover-action ${isBusy ? 'busy' : ''} ${(isInstalled || downloadStatus === 'completed') ? 'install-accent' : 'btn-primary'}`}
            onClick={handleAction}
            disabled={isBusy}
          >
            {isInstalling && <span className="btn-spinner" />}
            {getActionLabel()}
          </button>
        </div>
        {(downloadStatus === 'downloading' && downloadProgress !== undefined) && (
          <div className="card-inline-progress">
            <div className="card-inline-progress-fill" style={{ width: `${downloadProgress}%` }} />
          </div>
        )}
      </div>

      <div className="card-meta-top">
        <div className="card-title">{game.game_name || game.release_name}</div>
      </div>

      <div className="card-meta">{(!game.size || game.size === "0" || game.size === "0 MB") ? "Size unknown" : game.size} | v{game.version_code || ""}</div>

      {expanded && (
        <div className="card-expanded">
          <div className="card-expanded-block card-expanded-package">
            <span className="card-expanded-label">Package</span>
            <span className="card-expanded-value">{game.package_name}</span>
          </div>

          <div className="card-expanded-block">
             <span className="card-expanded-label">Downloads</span>
             <span className="card-expanded-value">{game.downloads}</span>
          </div>

          <div className="card-expanded-actions">
            <button
                type="button"
                className={`${actionClass}${isInstalling ? ' btn-installing' : ''}`}
                onClick={handleAction}
                disabled={isBusy}
            >
                {isInstalling && <span className="btn-spinner" />}
                {getActionLabel()}
            </button>
            <button
                type="button"
                className="btn-secondary btn-sm"
                onClick={(e) => { e.stopPropagation(); onDownload(game.package_name, selectedRelease); }}
                disabled={isBusy || downloadStatus === 'completed'}
            >
                Download Only
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
