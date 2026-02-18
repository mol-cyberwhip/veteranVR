/// <reference types="vite/client" />

interface Window {
  __TAURI__: {
    core: {
      invoke: (cmd: string, args?: any) => Promise<any>;
      convertFileSrc: (path: string) => string;
    };
  };
  __veteranFrontendBootstrapped: boolean;
  __veteranFrontendBootstrapWatchdog: number;
  __veteranReportFrontendError: (message: string, detail: string) => void;
  dismissUninstallDialog: () => void;
  confirmUninstall: () => Promise<void>;
  _downloadRemove: (pkg: string) => void;
  _downloadReorder: (pkg: string, pos: number) => void;
  _downloadRetry: (pkg: string) => void;
}

