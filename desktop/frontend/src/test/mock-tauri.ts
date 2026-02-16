
export function setupTauriMock() {
    console.log("[Mock] Setting up Tauri mock...");
    
    const mockGames: any[] = [
        {
            package_name: "com.beatgames.beatsaber",
            game_name: "Beat Saber",
            version_code: 2000,
            version_name: "1.35.0",
            size: "1.2 GB",
            is_favorite: false,
            is_new: false,
            popularity_rank: 1,
            downloads: "1.5M"
        },
        {
            package_name: "com.bonelab.game",
            game_name: "BONELAB",
            version_code: 1000,
            version_name: "1.0",
            size: "5 GB",
            is_favorite: true,
            is_new: true,
            popularity_rank: 2,
            downloads: "500K"
        }
    ];

    const mockDevice: any = {
        serial: "mock-serial-123",
        state: "device",
        model: "Mock Device",
        product: "Mock Product",
        is_selected: true,
        is_connected: true
    };

    const mockDeviceState: any = {
        status: "connected",
        status_message: "Connected to Mock Device",
        troubleshooting: "No issues detected",
        can_download: true,
        can_install: true,
        download_only_mode: false,
        selected_serial: "mock-serial-123",
        selection_source: "auto",
        devices: [mockDevice],
        storage: { total_mb: 64000, used_mb: 32000, free_mb: 32000 },
        battery: { level_percent: 100, status: "charging", is_charging: true, temperature_c: 25 },
        wireless: { saved_endpoint: null, auto_reconnect_enabled: false, last_endpoint: null, last_status: null, last_error: null, last_attempt_at: null },
        keep_awake: { enabled: false, interval_seconds: 60, active_count: 0, active_operation_ids: [] }
    };

    const mockCatalogStatus: any = {
        synced: true,
        source: "mock",
        game_count: mockGames.length,
        has_config: true,
        thumbnails_dir: "/tmp/thumbnails",
        notes_dir: "/tmp/notes",
        cache_dir: "/tmp/cache",
        sync_in_progress: false
    };

    const mockSettings: any = { 
        download_dir: "/tmp/downloads", 
        auto_install: true, 
        auto_backup: false, 
        backup_dir: "/tmp/backups", 
        theme: "dark", 
        language: "en", 
        enable_notifications: true, 
        concurrent_downloads: 1, 
        favorited_games: [], 
        wireless_auto_reconnect: false 
    };

    // Create the mock implementation
    const invokeMock = async (cmd: string, args: any) => {
        console.log(`[Mock] invoke: ${cmd}`, args);
        switch (cmd) {
            case 'backend_ready_state':
                return { 
                    status: "ready", 
                    ready: true, 
                    message: "Ready", 
                    backend_pid: 1234 
                };
            case 'backend_device_state':
                return mockDeviceState;
            case 'backend_catalog_status':
                return mockCatalogStatus;
            case 'backend_catalog_load_cache':
                 return mockCatalogStatus;
            case 'backend_catalog_library':
                return { 
                    games: mockGames, 
                    total: mockGames.length, 
                    offset: 0, 
                    limit: 500, 
                    query: args.query || "", 
                    sort_by: args.sortBy || "popularity", 
                    sort_ascending: args.sortAscending ?? true, 
                    filter: args.filter || "all", 
                    favorites_count: 1 
                };
            case 'backend_catalog_thumbnail_path':
                return {
                    thumbnail_exists: true,
                    thumbnail_path: `/tmp/thumbnails/${args.packageName}.png`
                };
            case 'backend_download_queue_status':
                return { queue: [], queued_count: 0, total_count: 0, processing: false };
            case 'backend_install_status':
                 return { history: [], history_count: 0, active_operation_id: null };
            case 'backend_installed_apps':
                return [];
            case 'backend_get_settings':
                return { settings: mockSettings };
            case 'backend_download_queue_add':
                return { status: "ok", result: { added: true } };
            case 'backend_download_start_processing':
                return { status: "ok", result: {} };
            default:
                console.warn(`[Mock] Unhandled command: ${cmd}`);
                return {};
        }
    };

    // Attach to window
    if (typeof window !== 'undefined') {
        window.__TAURI__ = {
            core: {
                invoke: invokeMock,
                convertFileSrc: (path: string) => `asset://${path}`
            }
        };
    }
}
