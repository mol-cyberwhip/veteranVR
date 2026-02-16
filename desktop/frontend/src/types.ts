export interface Game {
  package_name: string;
  game_name: string;
  release_name?: string;
  version_code: string | number;
  version_name?: string;
  size: string;
  last_updated?: string;
  is_favorite: boolean;
  is_new: boolean;
  popularity_rank: number;
  downloads: string | number;
  is_downloaded?: boolean;
}

export interface Device {
  serial: string;
  state: string;
  model: string;
  product: string;
  is_selected: boolean;
  is_connected: boolean;
}

export interface DeviceState {
  status: string;
  status_message: string;
  devices: Device[];
  connected: boolean; // Computed or from payload
  // ... add others as needed
}

export interface DownloadQueueItem {
    package_name: string;
    release_name: string;
    game_name?: string;
    status: string;
    progress_percent: number;
    speed: string;
    eta: string;
}
