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

  // Fetch trailer and notes when game changes
  useEffect(() => {
    if (!isOpen || !game) return;

    const fetchData = async () => {
      // Reset states
      setLoadingTrailer(true);
      setLoadingNotes(true);
      setTrailerId(null);
      setReleaseNotes('');

      try {
        // Fetch thumbnail
        const { exists, path } = await api.getThumbnailPath(game.package_name);
        if (exists && path) {
          setThumbnailSrc(api.convertFileSrc(path));
        }

        // Fetch trailer
        const videoId = await api.searchYoutubeTrailer(game.game_name);
        setTrailerId(videoId);
      } catch (e) {
        console.error('Failed to fetch trailer:', e);
      } finally {
        setLoadingTrailer(false);
      }

      try {
        // Fetch release notes
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

  // Handle escape key to close modal
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

  return (
    <div
      className="modal-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
    >
      <div className="modal-content">
        <button className="modal-close" onClick={onClose} aria-label="Close">
          ×
        </button>

        <div className="modal-header">
          <h2 className="modal-title">{game.game_name || game.release_name}</h2>
        </div>

        <div className="modal-grid">
          <div className="modal-info">
            <div className="modal-thumbnail">
              {thumbnailSrc ? (
                <img src={thumbnailSrc} alt={game.game_name} />
              ) : (
                <div className="modal-thumbnail-placeholder">
                  {(game.game_name || '?').substring(0, 2).toUpperCase()}
                </div>
              )}
            </div>

            <div className="modal-info-list">
              <div className="modal-info-item">
                <span className="modal-info-label">Package</span>
                <span className="modal-info-value">{game.package_name}</span>
              </div>
              <div className="modal-info-item">
                <span className="modal-info-label">Version</span>
                <span className="modal-info-value">{game.version_code}</span>
              </div>
              <div className="modal-info-item">
                <span className="modal-info-label">Size</span>
                <span className="modal-info-value">
                  {(!game.size || game.size === '0' || game.size === '0 MB')
                    ? 'Unknown'
                    : game.size}
                </span>
              </div>
              <div className="modal-info-item">
                <span className="modal-info-label">Downloads</span>
                <span className="modal-info-value">{game.downloads || '0'}</span>
              </div>
              {game.last_updated && (
                <div className="modal-info-item">
                  <span className="modal-info-label">Updated</span>
                  <span className="modal-info-value">{game.last_updated}</span>
                </div>
              )}
            </div>
          </div>

          <div className="modal-video">
            {loadingTrailer ? (
              <div className="modal-video-loading">
                <div className="loading-spinner" />
                <span>Finding trailer...</span>
              </div>
            ) : trailerId ? (
              <div className="video-container">
                <iframe
                  src={`https://www.youtube.com/embed/${trailerId}?autoplay=1&mute=1&rel=0`}
                  title={`${game.game_name} Trailer`}
                  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                  allowFullScreen
                />
              </div>
            ) : (
              <div className="modal-video-placeholder">
                <span>No trailer found</span>
              </div>
            )}
          </div>
        </div>

        {releaseNotes && (
          <div className="modal-notes">
            <h3 className="modal-section-title">Release Notes</h3>
            <div className="modal-notes-content">
              {loadingNotes ? (
                <div className="loading-spinner-small" />
              ) : (
                <pre>{releaseNotes}</pre>
              )}
            </div>
          </div>
        )}

        <div className="modal-actions">
          <button
            type="button"
            className={`btn-primary modal-action-btn ${isBusy ? 'busy' : ''}`}
            onClick={handlePrimaryAction}
            disabled={isBusy}
          >
            {isInstalling && <span className="btn-spinner" />}
            {getActionLabel()}
          </button>

          {!isInstalled && downloadStatus !== 'completed' && (
            <button
              type="button"
              className="btn-secondary modal-action-btn"
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
