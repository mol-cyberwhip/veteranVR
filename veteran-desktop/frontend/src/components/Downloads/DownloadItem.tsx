import React from 'react';
import { DownloadQueueItem, Game } from '../../types';
import styles from './DownloadItem.module.css';

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

  const statusClass = isCompleted ? styles['status-completed'] : isFailed ? styles['status-failed'] : isDownloading ? styles['status-downloading'] : isPaused ? styles['status-paused'] : '';

  return (
    <div className={`${styles['download-item']}${statusClass ? ` ${statusClass}` : ''}`}>
      <div className={styles['download-item-header']}>
        <span className={styles['download-item-name']}>{name}</span>
        <span className={styles['download-item-status']}>
            {isCompleted ? (
              <span className={styles['download-completed-badge']}>Ready to install</span>
            ) : (
              <>{item.status} {item.speed ? `(${item.speed})` : ''}</>
            )}
        </span>
      </div>

      {!isCompleted && (
        <div className={styles['download-item-progress-bar']}>
          <div
            className={styles['download-item-progress-fill']}
            style={{ width: `${item.progress_percent}%` }}
          />
        </div>
      )}

      <div className={styles['download-item-info']}>
        <span className={styles['download-item-percent']}>
          {isCompleted ? (
            <span className={styles['download-completed-size']}>{game?.size || 'Download complete'}</span>
          ) : (
            <>{item.progress_percent.toFixed(1)}%{item.eta ? ` - ETA: ${item.eta}` : ''}{isDownloading && item.speed ? ` - ${item.speed}` : ''}</>
          )}
        </span>
        <div className={styles['download-item-actions']}>
            {isDownloading && (
                <button
                    onClick={() => onPause(item.package_name)}
                    className="btn-sm btn-secondary"
                >
                    Pause
                </button>
            )}
            {isPaused && (
                <button
                    onClick={() => onResume(item.package_name)}
                    className="btn-sm btn-primary"
                >
                    Resume
                </button>
            )}
            {isCompleted && (
                <button
                    onClick={() => onInstall(item.package_name)}
                    className="btn-sm btn-success"
                >
                    Install
                </button>
            )}
            {isFailed && (
                <button
                    onClick={() => onRetry(item.package_name)}
                    className="btn-sm btn-primary"
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
