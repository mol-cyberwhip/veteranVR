pub mod ipc;
pub mod models;
pub mod services;
pub mod logger;

use ipc::commands::{register_invoke_handler, AppState};
use tauri::Manager;

pub fn run() {
    let app = register_invoke_handler(tauri::Builder::default())
        .setup(|app| {
            crate::services::binary_paths::init(app.handle());
            app.manage(AppState::new());

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                // Trigger catalog sync on startup
                let _ = crate::ipc::commands::backend_catalog_sync(state, Some(false)).await;
            });
            Ok(())
        })
        .on_window_event(|app, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let state = app.state::<AppState>();
                tauri::async_runtime::block_on(async {
                    let _ = state.rclone.shutdown().await;
                });
            }
        })
        .run(tauri::generate_context!());

    if let Err(err) = app {
        panic!("Failed to run tauri app: {err}");
    }
}
