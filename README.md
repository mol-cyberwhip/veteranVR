# Rookie Desktop - Build Instructions

## Quick Start

```bash
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

## See Also

- Full documentation: `desktop/README.md`
- Frontend code: `frontend/`
- Backend code: `src-tauri/src/`
