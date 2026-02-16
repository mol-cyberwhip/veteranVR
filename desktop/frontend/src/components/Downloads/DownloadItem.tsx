import React from 'react';
import { DownloadQueueItem, Game } from '../../types';

interface DownloadItemProps {
  item: DownloadQueueItem;
  game?: Game;
  onCancel: (pkg: string) => void;
  onInstall: (pkg: string) => void;
  onRetry: (pkg: string) => void;
  onPause: (pkg: string) => void;
  onResume: (pkg: string) => void;
}

export const DownloadItem: React.FC<DownloadItemProps> = ({ item, game, onCancel, onInstall, onRetry, onPause, onResume }) => {
  const name = item.game_name || game?.game_name || item.package_name;
  const isCompleted = item.status === 'completed';
  const isFailed = item.status === 'failed';
  const isDownloading = item.status === 'downloading';
  const isPaused = item.status === 'paused';

  const statusClass = isCompleted ? 'status-completed' : isFailed ? 'status-failed' : isDownloading ? 'status-downloading' : isPaused ? 'status-paused' : '';

  return (
    <div className={`download-item ${statusClass}`}>
      <div className="download-item-header">
        <span className="download-item-name">{name}</span>
        <span className="download-item-status">
            {item.status} {item.speed ? `(${item.speed})` : ''}
        </span>
      </div>

      <div className="download-item-progress-bar">
        <div
          className="download-item-progress-fill"
          style={{ width: `${item.progress_percent}%` }}
        />
      </div>

      <div className="download-item-info">
        <span className="download-item-percent">
          {item.progress_percent.toFixed(1)}%{item.eta ? ` - ETA: ${item.eta}` : ''}
        </span>
        <div className="download-item-actions">
            {isDownloading && (
                <button
                    onClick={() => onPause(item.package_name)}
                    className="btn-sm"
                >
                    Pause
                </button>
            )}
            {isPaused && (
                <button
                    onClick={() => onResume(item.package_name)}
                    className="btn-sm"
                >
                    Resume
                </button>
            )}
            {isCompleted && (
                <button
                    onClick={() => onInstall(item.package_name)}
                    className="btn-sm install-accent"
                >
                    Install
                </button>
            )}
            {isFailed && (
                <button
                    onClick={() => onRetry(item.package_name)}
                    className="btn-sm"
                >
                    Retry
                </button>
            )}
            {!isCompleted && (
                <button
                    onClick={() => onCancel(item.package_name)}
                    className="btn-sm btn-danger"
                >
                    Cancel
                </button>
            )}
            {isCompleted && (
                 <button
                    onClick={() => onCancel(item.package_name)}
                    className="btn-sm btn-ghost"
                    title="Remove from history"
                >
                    ✕
                </button>
            )}
        </div>
      </div>
    </div>
  );
};
