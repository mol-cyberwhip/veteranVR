import { useState, useEffect, useCallback } from 'react';
import { Game } from '../../types';
import { api } from '../../services/api';

interface GameDetailModalProps {
  game: Game;
  isOpen: boolean;
  onClose: () => void;
  onDownload: (pkg: string, releaseName?: string) => void;
  onInstall: (pkg: string, releaseName?: string) => void;
  isInstalled: boolean;
  hasUpdate: boolean;
  downloadStatus?: string;
  downloadProgress?: number;
  installStatus?: string;
}

export const GameDetailModal: React.FC<GameDetailModalProps> = ({
  game,
  isOpen,
  onClose,
  onDownload,
  onInstall,
  isInstalled,
  hasUpdate,
  downloadStatus,
  downloadProgress,
  installStatus,
}) => {
  const [trailerId, setTrailerId] = useState<string | null>(null);
  const [loadingTrailer, setLoadingTrailer] = useState(true);
  const [thumbnailSrc, setThumbnailSrc] = useState<string | null>(null);
  const [releaseNotes, setReleaseNotes] = useState<string>('');
  const [loadingNotes, setLoadingNotes] = useState(true);

  useEffect(() => {
    if (!isOpen || !game) return;

    const fetchData = async () => {
      setLoadingTrailer(true);
      setLoadingNotes(true);
      setTrailerId(null);
      setReleaseNotes('');

      try {
        const { exists, path } = await api.getThumbnailPath(game.package_name);
        if (exists && path) {
          setThumbnailSrc(api.convertFileSrc(path));
        }

        const videoId = await api.searchYoutubeTrailer(game.game_name);
        setTrailerId(videoId);
      } catch (e) {
        console.error('Failed to fetch trailer:', e);
      } finally {
        setLoadingTrailer(false);
      }

      try {
        const noteData = await api.getGameNote(game.package_name);
        if (noteData && noteData.note) {
          setReleaseNotes(noteData.note);
        }
      } catch (e) {
        console.error('Failed to fetch release notes:', e);
      } finally {
        setLoadingNotes(false);
      }
    };

    fetchData();
  }, [game, isOpen]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose]
  );

  useEffect(() => {
    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = '';
    };
  }, [isOpen, handleKeyDown]);

  if (!isOpen || !game) return null;

  const isInstalling = !!installStatus;
  const isBusy = isInstalling || downloadStatus === 'downloading' || downloadStatus === 'queued';

  const getActionLabel = () => {
    if (isInstalling) return installStatus!;
    if (downloadStatus === 'downloading') return `Downloading ${downloadProgress?.toFixed(0)}%`;
    if (downloadStatus === 'queued') return 'Pending...';
    if (isInstalled && hasUpdate) return 'Update';
    if (isInstalled) return 'Reinstall';
    if (downloadStatus === 'completed') return 'Install';
    return 'Download & Install';
  };

  const handlePrimaryAction = () => {
    if (isInstalled && !hasUpdate) {
      onInstall(game.package_name, game.release_name);
    } else if (isInstalled && hasUpdate) {
      onInstall(game.package_name, game.release_name);
    } else if (downloadStatus === 'completed') {
      onInstall(game.package_name, game.release_name);
    } else {
      onDownload(game.package_name, game.release_name);
    }
  };

  const handleDownloadOnly = () => {
    onDownload(game.package_name, game.release_name);
  };

  const fallbackLabel = (game.game_name || '?').substring(0, 2).toUpperCase();

  return (
    <div
      className="modal-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
    >
      <div className="modal-container">
        {/* Header */}
        <div className="modal-header-row">
          <h1 className="modal-game-title">{game.game_name || game.release_name}</h1>
          <button className="modal-close-btn" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        {/* Main Content Grid */}
        <div className="modal-body">
          {/* Left Column */}
          <div className="modal-col-left">
            <div className="modal-media-thumb">
              {thumbnailSrc ? (
                <img src={thumbnailSrc} alt={game.game_name} />
              ) : (
                <div className="modal-thumb-placeholder">{fallbackLabel}</div>
              )}
            </div>
            
            <div className="modal-meta-panel">
              <div className="meta-row">
                <span className="meta-key">Package</span>
                <span className="meta-val" title={game.package_name}>{game.package_name}</span>
              </div>
              <div className="meta-row">
                <span className="meta-key">Version</span>
                <span className="meta-val">{game.version_code}</span>
              </div>
              <div className="meta-row">
                <span className="meta-key">Size</span>
                <span className="meta-val">
                  {(!game.size || game.size === '0' || game.size === '0 MB') ? 'Unknown' : game.size}
                </span>
              </div>
              <div className="meta-row">
                <span className="meta-key">Downloads</span>
                <span className="meta-val">{game.downloads || '0'}</span>
              </div>
              {game.last_updated && (
                <div className="meta-row">
                  <span className="meta-key">Updated</span>
                  <span className="meta-val">{game.last_updated}</span>
                </div>
              )}
            </div>
          </div>

          {/* Right Column */}
          <div className="modal-col-right">
            <div className="modal-video-wrapper">
              {loadingTrailer ? (
                <div className="video-loading-state">
                  <div className="spinner" />
                  <span>Finding trailer...</span>
                </div>
              ) : trailerId ? (
                <iframe
                  src={`https://www.youtube.com/embed/${trailerId}?autoplay=1&mute=1&rel=0`}
                  title={`${game.game_name} Trailer`}
                  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                  allowFullScreen
                />
              ) : (
                <div className="video-empty-state">
                  <span>No trailer found</span>
                </div>
              )}
            </div>

            {releaseNotes && (
              <div className="modal-notes-section">
                <h3 className="notes-title">Release Notes</h3>
                <div className="notes-content">
                  {loadingNotes ? (
                    <div className="spinner-small" />
                  ) : (
                    <pre>{releaseNotes}</pre>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer Actions */}
        <div className="modal-footer">
          <button
            type="button"
            className={`action-btn-primary ${isBusy ? 'busy' : ''}`}
            onClick={handlePrimaryAction}
            disabled={isBusy}
          >
            {isInstalling && <span className="btn-spinner" />}
            {getActionLabel()}
          </button>

          {!isInstalled && downloadStatus !== 'completed' && (
            <button
              type="button"
              className="action-btn-secondary"
              onClick={handleDownloadOnly}
              disabled={isBusy || downloadStatus === 'completed'}
            >
              Download Only
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
