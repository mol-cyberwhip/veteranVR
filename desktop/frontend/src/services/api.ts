import {
  assertCatalogStatusSnapshot,
  assertDeviceStateSnapshot,
  assertLibraryResult,
  assertDownloadQueueStatus,
  assertInstallStatusResult
} from "../lib/contract";

const invoke = async (cmd: string, args?: any) => {
  if (!window.__TAURI__ || !window.__TAURI__.core) {
    throw new Error("Tauri bridge unavailable - app must run inside Tauri");
  }
  const payload = await window.__TAURI__.core.invoke(cmd, args);
  // Unwrap envelope if present
  return (payload && payload.status === 'ok' && payload.result !== undefined) ? payload.result : payload;
};

export const api = {
  getReadyState: async () => {
    const payload = await invoke("backend_ready_state");
    return payload; 
  },
  
  getDeviceState: async (refresh = true) => {
    const payload = await invoke("backend_device_state", { refresh });
    return assertDeviceStateSnapshot(payload);
  },

  getCatalogStatus: async () => {
    const payload = await invoke("backend_catalog_status");
    return assertCatalogStatusSnapshot(payload);
  },

  loadCache: async () => {
    const payload = await invoke("backend_catalog_load_cache");
    return assertCatalogStatusSnapshot(payload);
  },

  syncCatalog: async () => {
    const payload = await invoke("backend_catalog_sync");
    return assertCatalogStatusSnapshot(payload);
  },

  getLibrary: async (query = "", sortBy = "popularity", sortAscending = true, filter = "all", limit = 500, offset = 0) => {
    const payload = await invoke("backend_catalog_library", {
      query,
      sortBy,
      sortAscending,
      filter,
      limit,
      offset,
    });
    return assertLibraryResult(payload);
  },

  getThumbnailPath: async (packageName: string) => {
    const payload = await invoke("backend_catalog_thumbnail_path", { packageName });
    return {
        exists: !!payload?.thumbnail_exists,
        path: payload?.thumbnail_path ? String(payload.thumbnail_path) : ""
    };
  },

  convertFileSrc: (path: string) => {
    if (window.__TAURI__?.core?.convertFileSrc) {
        return window.__TAURI__.core.convertFileSrc(path);
    }
    return `file://${path}`;
  },

  getDownloadQueue: async () => {
    const payload = await invoke("backend_download_queue_status");
    return assertDownloadQueueStatus(payload);
  },

  getInstalledApps: async () => {
    const payload = await invoke("backend_installed_apps", { includeUpdateDetection: true });
    return payload; 
  },

  getInstallStatus: async () => {
    const payload = await invoke("backend_install_status");
    return assertInstallStatusResult(payload);
  },

  getBackups: async () => {
    return invoke("backend_list_backups");
  },

  // Actions
  queueDownload: async (packageName: string, releaseName?: string) => {
    return invoke("backend_download_queue_add", { packageName, releaseName });
  },

  startDownloadProcessing: async () => {
    return invoke("backend_download_start_processing");
  },

  pauseDownload: async () => {
    return invoke("backend_download_pause");
  },

  resumeDownload: async () => {
    return invoke("backend_download_resume");
  },
  
  cancelDownload: async (packageName: string) => {
    return invoke("backend_download_cancel", { packageName });
  },

  installGame: async (packageName: string, releaseName?: string) => {
    return invoke("backend_install_game", { packageName, releaseName });
  },

  uninstallGame: async (packageName: string, keepObb = false, keepData = false) => {
    return invoke("backend_uninstall_game", { packageName, keepObb, keepData });
  },

  backupApp: async (packageName: string, includeObb = true) => {
    return invoke("backend_backup_save_data", { packageName, includeObb });
  },

  restoreApp: async (packageName: string, includeObb = true) => {
    return invoke("backend_restore_save_data", { packageName, includeObb });
  },

  openDownloadFolder: async () => {
    return invoke("backend_download_open_folder", { packageName: null });
  },

  recoverBackend: async () => {
    return invoke("backend_recover");
  },

  pollEvents: async (limit = 100) => {
    return invoke("poll_backend_events", { operationId: null, limit });
  }
};
