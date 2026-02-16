import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../services/api';
import { Game } from '../types';

interface AppContextType {
  deviceStatus: any;
  catalogStatus: any;
  downloadQueue: any;
  installedApps: any[];
  gameMap: Map<string, Game>;
  installingPackages: Map<string, string>; // pkg -> status message
  uninstallingPackages: Set<string>;
  refreshDevice: () => Promise<void>;
  syncCatalog: () => Promise<void>;
  refreshLibraryMap: () => Promise<void>;
  refreshQueue: () => Promise<void>;
  startInstall: (pkg: string, releaseName?: string) => Promise<void>;
  startUninstall: (pkg: string) => Promise<void>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [catalogStatus, setCatalogStatus] = useState<any>(null);
  const [downloadQueue, setDownloadQueue] = useState<any>(null);
  const [installedApps, setInstalledApps] = useState<any[]>([]);
  const [gameMap, setGameMap] = useState<Map<string, Game>>(new Map());
  const [installingPackages, setInstallingPackages] = useState<Map<string, string>>(new Map());
  const [uninstallingPackages, setUninstallingPackages] = useState<Set<string>>(new Set());
  const installingOpsRef = useRef<Map<string, string>>(new Map()); // operationId -> packageName

  // Poll Device State
  useEffect(() => {
    const fetchDevice = async () => {
        try {
            const status = await api.getDeviceState(true);
            setDeviceStatus(status);
        } catch (e) {
            console.error("Device fetch failed", e);
        }
    };

    fetchDevice();
    const interval = setInterval(fetchDevice, 2000);
    return () => clearInterval(interval);
  }, []);

  // Poll Catalog Status
  useEffect(() => {
    const fetchCatalog = async () => {
        try {
            const status = await api.getCatalogStatus();
            setCatalogStatus(status);
        } catch (e) {
            console.error("Catalog status fetch failed", e);
        }
    };

    fetchCatalog();
    const interval = setInterval(fetchCatalog, 1500);
    return () => clearInterval(interval);
  }, []);

  // Poll Download Queue
  const refreshQueue = React.useCallback(async () => {
      try {
          const queue = await api.getDownloadQueue();
          setDownloadQueue(queue);
      } catch (e) { console.error("Queue fetch failed", e); }
  }, []);

  useEffect(() => {
      refreshQueue();
      const interval = setInterval(refreshQueue, 1000);
      return () => clearInterval(interval);
  }, [refreshQueue]);

  // Poll Installed Apps (less frequent)
  useEffect(() => {
      const fetchApps = async () => {
          try {
             const result = await api.getInstalledApps();
             const apps = result?.apps ?? result;
             setInstalledApps(Array.isArray(apps) ? apps : []);
          } catch (e) { console.error("Installed apps fetch failed", e); }
      };
      fetchApps();
      const interval = setInterval(fetchApps, 5000);
      return () => clearInterval(interval);
  }, []);

  // Initial Library Map Population
  const refreshLibraryMap = async () => {
      try {
          const result = await api.getLibrary("", "popularity", true, "all", 2000, 0);
          const newMap = new Map(gameMap);
          result.games.forEach((g: Game) => newMap.set(g.package_name, g));
          setGameMap(newMap);
      } catch (e) { console.error("Library map fetch failed", e); }
  };

  useEffect(() => {
      refreshLibraryMap();
  }, [catalogStatus?.synced]);

  const refreshDevice = async () => {
    try {
        const status = await api.getDeviceState(true);
        setDeviceStatus(status);
    } catch (e) { console.error(e); }
  };

  const syncCatalog = async () => {
    try {
        await api.syncCatalog();
    } catch (e) { console.error(e); }
  };

  const startInstall = useCallback(async (pkg: string, releaseName?: string) => {
    setInstallingPackages(prev => new Map(prev).set(pkg, 'Installing...'));
    try {
      const result = await api.installGame(pkg, releaseName);
      if (result?.operation_id) {
        installingOpsRef.current.set(result.operation_id, pkg);
      }
    } catch (e: any) {
      setInstallingPackages(prev => {
        const next = new Map(prev);
        next.delete(pkg);
        return next;
      });
      throw e;
    }
  }, []);

  const startUninstall = useCallback(async (pkg: string) => {
    setUninstallingPackages(prev => new Set(prev).add(pkg));
    try {
      await api.uninstallGame(pkg);
      await refreshDevice();
    } finally {
      setUninstallingPackages(prev => {
        const next = new Set(prev);
        next.delete(pkg);
        return next;
      });
    }
  }, []);

  // Watch for install completion events via polling
  useEffect(() => {
    if (installingPackages.size === 0) return;

    const checkInstallEvents = async () => {
      try {
        const result = await api.pollEvents(50);
        const events = result?.events;
        if (!Array.isArray(events)) return;

        for (const ev of events) {
          if (!ev?.operation) continue;
          const opId = ev.operation.operation_id;
          const pkg = installingOpsRef.current.get(opId);
          if (!pkg) continue;

          if (ev.operation.terminal) {
            installingOpsRef.current.delete(opId);
            setInstallingPackages(prev => {
              const next = new Map(prev);
              next.delete(pkg);
              return next;
            });
            // Refresh installed apps to reflect the change
            refreshDevice();
          } else if (ev.message) {
            setInstallingPackages(prev => new Map(prev).set(pkg, ev.message));
          }
        }
      } catch { /* ignore */ }
    };

    const interval = setInterval(checkInstallEvents, 1500);
    return () => clearInterval(interval);
  }, [installingPackages.size]);

  // Safety timeout: clear stale installing states after 5 minutes
  useEffect(() => {
    if (installingPackages.size === 0) return;
    const timeout = setTimeout(() => {
      setInstallingPackages(new Map());
      installingOpsRef.current.clear();
    }, 5 * 60 * 1000);
    return () => clearTimeout(timeout);
  }, [installingPackages.size]);

  return (
    <AppContext.Provider value={{
        deviceStatus,
        catalogStatus,
        downloadQueue,
        installedApps,
        gameMap,
        installingPackages,
        uninstallingPackages,
        refreshDevice,
        syncCatalog,
        refreshLibraryMap,
        refreshQueue,
        startInstall,
        startUninstall,
    }}>
      {children}
    </AppContext.Provider>
  );
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) throw new Error("useApp must be used within AppProvider");
  return context;
};
