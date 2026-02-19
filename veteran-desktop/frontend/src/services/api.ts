import {
  assertCatalogStatusSnapshot,
  assertDeviceStateSnapshot,
  assertLibraryResult,
  assertDownloadQueueStatus,
  assertInstallStatusResult
} from "../lib/contract";
import { commands } from "../bindings";

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
  
  getDeviceState: async () => {
    const result = await commands.backendDeviceState();
    if (result.status === "error") throw new Error(result.error);
    return assertDeviceStateSnapshot(result.data);
  },

  getCatalogStatus: async () => {
    const payload = await invoke("backend_catalog_status");
    return assertCatalogStatusSnapshot(payload);
  },

  loadCache: async () => {
    const payload = await invoke("backend_catalog_load_cache");
    return assertCatalogStatusSnapshot(payload);
  },

  syncCatalog: async (force = true) => {
    const payload = await invoke("backend_catalog_sync", { force });
    return assertCatalogStatusSnapshot(payload);
  },

  searchYoutubeTrailer: async (gameName: string) => {
    const result = await commands.searchYoutubeTrailer(gameName);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
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
    const result = await commands.backendCatalogThumbnailPath(packageName);
    if (result.status === "error") throw new Error(result.error);
    const payload = result.data;
    return {
        exists: !!payload?.thumbnail_exists,
        path: payload?.thumbnail_path ? String(payload.thumbnail_path) : ""
    };
  },

  getGameNote: async (packageName: string) => {
    const result = await commands.backendCatalogNote(packageName);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
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
    const result = await commands.backendInstalledApps();
    if (result.status === "error") throw new Error(result.error);
    return result.data;
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

  pauseItem: async (packageName: string) => {
    return invoke("backend_download_pause_item", { packageName });
  },

  resumeItem: async (packageName: string) => {
    return invoke("backend_download_resume_item", { packageName });
  },
  
  cancelDownload: async (packageName: string) => {
    return invoke("backend_download_cancel", { packageName });
  },

  installGame: async (packageName: string, releaseName: string | null = null) => {
    const result = await commands.backendInstallGame(packageName, releaseName);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },

  uninstallGame: async (packageName: string, keepObb = false, keepData = false) => {
    const result = await commands.backendUninstallGame(packageName, keepObb, keepData);
    if (result.status === "error") throw new Error(result.error);
    if (!result.data.uninstalled) {
      throw new Error(result.data.message || "Uninstall failed");
    }
    return result.data;
  },

  installLocalApk: async (path: string) => {
    const apkPath = path.trim();
    if (!apkPath) {
      throw new Error("Select an APK file first.");
    }
    if (!apkPath.toLowerCase().endsWith(".apk")) {
      throw new Error("Only .apk files can be sideloaded.");
    }

    const result = await commands.backendInstallLocal(apkPath);
    if (result.status === "error") throw new Error(result.error);
    if (!result.data.success) {
      throw new Error(result.data.message || "Local APK install failed");
    }
    return result.data;
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
  },

  wirelessEnableTcpip: async () => {
    const result = await commands.backendWirelessEnableTcpip(null);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },

  wirelessConnect: async (endpoint: string, saveEndpoint = true) => {
    const result = await commands.backendWirelessConnect(endpoint, saveEndpoint);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },

  wirelessDisconnect: async (endpoint?: string) => {
    const result = await commands.backendWirelessDisconnect(endpoint ?? null);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },

  wirelessReconnect: async () => {
    const result = await commands.backendWirelessReconnect();
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },

  wirelessScan: async (subnet?: string) => {
    const result = await commands.backendWirelessScan(subnet ?? null);
    if (result.status === "error") throw new Error(result.error);
    return result.data;
  },
};
