# Veteran Desktop (Tauri v2)

Veteran Desktop is a cross-platform VR sideloading application built with **Rust** and **Tauri v2**. It provides a high-performance, streamlined interface for managing, downloading, and installing VR content onto Android-based VR headsets like the Meta Quest.

> **Note:** This project has been migrated from a Python backend to a native Rust architecture for improved performance, better resource management, and easier distributability.

## 🚀 Key Features

-   **Device Management**:
    -   Automatic device detection via ADB.
    -   Wireless ADB connection with auto-reconnect support.
    -   View device status including battery levels, storage details, and connection state.
    -   Selected device context for targeted operations.
-   **Game Catalog**:
    -   Browsable library of VR titles with rich metadata (thumbnails, release notes).
    -   High-performance search, filtering, and sorting (by name, date, size, popularity).
    -   Background metadata synchronization with 4-hour caching logic.
-   **Content Management**:
    -   High-speed downloads via **Rclone** remote control (RCD) API.
    -   Queue management with support for reordering and cancellation.
    -   Automatic extraction of 7z archives with password support.
    -   Smart installation of APKs and OBB files.
-   **System & Library**:
    -   View and manage installed applications.
    -   App uninstallation and local APK installation.
    -   Media file management and transfer.
    -   App backup and restore capabilities.
-   **Diagnostics & Logs**:
    -   Internal logging system with export and upload capabilities.
    -   Device performance profiling.
    -   Interactive ADB console.

## 🏗 Architecture

The application is structured as a modern [Tauri](https://tauri.app/) project:

### Backend (Rust)
Located in `src-tauri/`, the backend handles all heavy-lifting services:
-   **IPC Commands** (`src/ipc/commands.rs`): Defines all frontend-callable functions. Uses [Specta](https://github.com/oscartbeaumont/specta) for automated TypeScript binding generation.
-   **Services** (`src/services/`):
    -   `adb.rs`: Wraps `adb_client` and external `adb` CLI for device interaction.
    -   `catalog.rs`: Manages the game library, metadata parsing, and search.
    -   `download.rs`: Core download queue and status tracking.
    -   `rclone.rs`: Manages the lifecycle of an `rclone rcd` daemon and communicates with its HTTP API.
    -   `install.rs`: Orchestrates APK installation, OBB placement, and archive extraction.
    -   `settings.rs`: Persistent configuration management using JSON.
-   **Models** (`src/models/`): Strongly-typed data structures shared across services and IPC.

### Frontend (React)
Located in `frontend/`, the frontend is a modern React application:
-   **React 19 + TypeScript**: Built with Vite for fast development.
-   **Tauri-Specta Bindings**: Type-safe IPC client generated from the Rust commands.
-   **State Management**: Centralized `AppContext` for application-wide state (devices, downloads, settings).
-   **Modular UI**: Component-based views for Library, Downloads, Backups, and Diagnostics.

## 🛠 Prerequisites

-   **Rust**: Latest stable release.
-   **Node.js**: LTS version for frontend builds.
-   **ADB**: Android Debug Bridge must be installed and accessible in your system `PATH`.
-   **Rclone**: Required for fetching content; must be in your system `PATH`.
-   **7-Zip (Optional)**: Used as a fallback for some extraction tasks.

## 💻 Development

1.  **Clone the repository**.
2.  **Install frontend dependencies**:
    ```bash
    cd frontend
    npm install
    ```
3.  **Run in development mode** (from the root directory):
    ```bash
    cargo tauri dev
    ```
    This command will:
    -   Start the Vite dev server for the frontend.
    -   Compile the Rust backend.
    -   Launch the application window with HMR enabled.

4.  **Update Type Bindings**:
    The project uses [tauri-specta](https://github.com/oscartbeaumont/specta) to automatically generate TypeScript bindings from Rust code. **This does NOT happen automatically during `cargo build`** - you must manually regenerate after modifying Rust commands.
    
    To regenerate bindings:
    ```bash
    cd src-tauri
    cargo test generate_bindings
    ```
    This will regenerate `src/bindings.ts` (also copied to `frontend/src/bindings.ts`).
    
    **Important**: Always regenerate bindings when you:
    - Add, remove, or rename IPC commands in `src/ipc/commands.rs`
    - Change command argument types or return types
    - Modify shared data structures in `src/models/`
    
    The generated bindings provide full type safety between Rust and TypeScript.

## 📝 TypeScript Development

The frontend uses TypeScript with Vite for fast development and type checking:

-   **Type Checking**: Run `npx tsc --noEmit` in the `frontend/` directory to check types without emitting files
-   **Development Server**: `npm run dev` starts the Vite dev server with HMR (Hot Module Replacement)
-   **Build**: `npm run build` runs type checking (`tsc`) followed by the Vite build

**Configuration** (`frontend/tsconfig.json`):
-   Target: ES2020
-   Module: ESNext with bundler resolution
-   JSX: react-jsx transform
-   Strict mode enabled
-   Note: `noUnusedLocals` and `noUnusedParameters` are disabled to accommodate auto-generated bindings
-   Path aliases: `@/*` → `./src/*`, `@bindings` → `./src/bindings.ts`

## 📂 Codebase Overview

```text
.
├── frontend/               # React frontend source
│   ├── src/
│   │   ├── components/     # UI views and widgets
│   │   ├── context/        # State management (AppContext)
│   │   ├── services/       # API wrappers
│   │   └── bindings.ts     # Auto-generated IPC client
├── src-tauri/              # Rust backend source
│   ├── src/
│   │   ├── ipc/            # Bridge logic and commands.rs
│   │   ├── models/         # Data structures
│   │   ├── services/       # Core business logic (ADB, Rclone, etc.)
│   │   └── lib.rs          # App lifecycle and setup
│   └── Cargo.toml          # Backend dependencies
└── tauri.conf.json         # Tauri configuration (v2)
```

## ⚙️ Configuration

-   **Archive Password**: The default password for encrypted content is managed via the public configuration service and defaults to `gL59VfgPxoHR`.
-   **Storage**: Application data (cache, settings, logs) is stored in `$HOME/.veteran/`.
-   **Downloads**: Default download path is configurable in the application settings.

## 🔨 Build Scripts

For convenience, several build scripts are provided:

-   **`./scripts/build.sh`**: Complete build pipeline that runs tests, regenerates bindings, builds the frontend, and builds the Tauri application.
-   **`./scripts/build-all.sh`**: Cross-compilation script for building all platforms (macOS, Linux, Windows).
-   **`./scripts/download-binaries.sh`**: Downloads sidecar binaries (adb, rclone, 7z) for production builds.

## 🤝 Contributing

When adding new features:
1.  Implement the logic in a new or existing service in `src-tauri/src/services/`.
2.  Expose the service via a command in `src-tauri/src/ipc/commands.rs`.
3.  Run `cargo test --lib generate_bindings` to sync the frontend client.
4.  Copy the updated bindings: `cp src/bindings.ts frontend/src/bindings.ts`
5.  Implement the UI component in `frontend/src/components/` and wire it up in `App.tsx` or its respective view.
