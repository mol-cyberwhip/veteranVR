# Rookie Desktop - Build Instructions

**Important**: All commands in this document should be run from the `desktop/` directory unless otherwise specified.

## Quick Start

```bash
# Navigate to the desktop directory
cd desktop

# Install dependencies
cd frontend && npm install && cd ..

# Development
cargo tauri dev

# Production build (macOS only)
./scripts/download-binaries.sh
cargo tauri build
```

## Cross-Platform Builds

Build for all platforms:

```bash
./scripts/build-all.sh
```

Or specific targets:
```bash
./scripts/build-all.sh mac-arm      # macOS Apple Silicon
./scripts/build-all.sh mac-x86      # macOS Intel
./scripts/build-all.sh linux        # Linux x86_64
./scripts/build-all.sh windows      # Windows x86_64
```

### Prerequisites for Cross-Compilation (from macOS)

- **Linux**: `brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu`
- **Windows**: `cargo install cargo-xwin`

## Sidecar Binaries

This app bundles `adb`, `rclone`, and `7z` as sidecars:

1. **Development**: Uses system binaries from `PATH`
2. **Production**: Downloads and bundles platform-specific binaries

### Download Binaries for Production

```bash
./scripts/download-binaries.sh [target-triple]
```

Binaries are stored in `src-tauri/binaries/` and bundled via Tauri's `externalBin` feature.

## Output

Builds are output to:
- `src-tauri/target/release/bundle/` (per-target subdirectories)
- `dist/<target>/` (copied by build-all.sh)

## TypeScript Bindings (Critical!)

This project uses [tauri-specta](https://github.com/oscartbeaumont/specta) to generate TypeScript bindings from Rust code. **Bindings do NOT regenerate automatically during build** - you must manually regenerate them.

### When to Regenerate Bindings

You MUST regenerate bindings when you modify:
- IPC commands in `src/ipc/commands.rs` (add/remove/rename commands)
- Command argument types or return types
- Shared data structures in `src/models/`
- Any type with `specta::Type` derive that is exposed to the frontend

### How to Regenerate Bindings

```bash
# 1. Run the binding generation test
cd src-tauri
cargo test --lib generate_bindings

# 2. Copy the generated bindings to the frontend
cp ../src/bindings.ts ../frontend/src/bindings.ts

# Or use the build script (see below) which does this automatically
```

### Common Issues

**Duplicate type errors in bindings.ts**: This happens when the same type name is defined in multiple places. The build will fail with "Duplicate identifier" errors. To fix:
1. Check for duplicate type definitions in `src/models/`
2. Ensure internal types don't conflict with API response types
3. Rename internal types if necessary (e.g., `DeviceInfo` → `RawDeviceInfo`)

**"Cannot find name" errors in TypeScript**: The bindings are out of sync with the Rust code. Regenerate them using the steps above.

### Full Development Workflow

```bash
# 1. Make changes to Rust code
# 2. Regenerate bindings
cd src-tauri
cargo test --lib generate_bindings
cp ../src/bindings.ts ../frontend/src/bindings.ts

# 3. Run Rust tests
cargo test

# 4. Build and test frontend
cd ../frontend
npm run build

# 5. Run full application
cd ..
cargo tauri dev
```

### Automated Build Script

For convenience, use the provided build script:

```bash
./scripts/build.sh
```

This script will:
1. Run all Rust tests
2. Regenerate TypeScript bindings
3. Copy bindings to frontend
4. Build the frontend
5. Build the Tauri application (skipping redundant frontend build)

**Note**: The script uses `--no-before-build-command` to avoid running the frontend build twice (we already build it in Step 4).

See `desktop/scripts/build.sh` for details.

## See Also

- Full documentation: `desktop/README.md`
- Frontend code: `frontend/`
- Backend code: `src-tauri/src/`
