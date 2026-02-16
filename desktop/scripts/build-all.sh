#!/usr/bin/env bash
set -euo pipefail

# Build Veteran Desktop for all major platforms.
# Compatible with Bash 3.2+ (macOS default)
#
# Usage:
#   ./scripts/build-all.sh                  # build all 4 targets
#   ./scripts/build-all.sh mac-arm          # build one target
#   ./scripts/build-all.sh mac-arm mac-x86  # build selected targets
#
# Supported target aliases:
#   mac-arm    -> aarch64-apple-darwin
#   mac-x86    -> x86_64-apple-darwin
#   linux      -> x86_64-unknown-linux-gnu
#   linux-arm  -> aarch64-unknown-linux-gnu
#   windows    -> x86_64-pc-windows-msvc
#
# Prerequisites:
#   - Rust cross-compilation toolchains installed for each target:
#       rustup target add aarch64-apple-darwin x86_64-apple-darwin \
#                         x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc
#   - For Linux x86_64 from macOS: a cross-linker (e.g. via `brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu`)
#   - For Linux ARM from macOS: a cross-linker (e.g. via `brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu`)
#   - For Windows from macOS: cargo-xwin (`cargo install cargo-xwin`) or cross (`cargo install cross`)
#   - Tauri CLI: `cargo install tauri-cli` (or `npx @tauri-apps/cli`)
#   - Node.js + npm (for frontend build)

# Ensure cargo bin is in PATH (don't source shell configs - they may hang in non-interactive shells)
export PATH="$HOME/.cargo/bin:$PATH:/opt/homebrew/bin:/usr/local/bin"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DOWNLOAD_SCRIPT="$SCRIPT_DIR/download-binaries.sh"
DIST_DIR="$PROJECT_DIR/dist"

# Target definitions - use function instead of associative array for Bash 3.2 compatibility
get_target_triple() {
    case "$1" in
        mac-arm)   echo "aarch64-apple-darwin" ;;
        mac-x86)   echo "x86_64-apple-darwin" ;;
        linux)     echo "x86_64-unknown-linux-gnu" ;;
        linux-arm) echo "aarch64-unknown-linux-gnu" ;;
        windows)   echo "x86_64-pc-windows-msvc" ;;
        *)         echo "" ;;
    esac
}

is_valid_target() {
    case "$1" in
        mac-arm|mac-x86|linux|linux-arm|windows) return 0 ;;
        *) return 1 ;;
    esac
}

ALL_ALIASES=(mac-arm mac-x86 linux linux-arm windows)

# Parse arguments
REQUESTED=()
SKIP_BINARIES=false
VERBOSE=false

for arg in "$@"; do
    case "$arg" in
        --skip-binaries) SKIP_BINARIES=true ;;
        --verbose|-v) VERBOSE=true ;;
        --help|-h)
            echo "Build Veteran Desktop for multiple platforms"
            echo ""
            echo "Usage:"
            echo "  ./scripts/build-all.sh [options] [target...]"
            echo ""
            echo "Targets:"
            echo "  mac-arm     macOS Apple Silicon (aarch64-apple-darwin)"
            echo "  mac-x86     macOS Intel (x86_64-apple-darwin)"
            echo "  linux       Linux x86_64 (x86_64-unknown-linux-gnu)"
            echo "  linux-arm   Linux ARM64 (aarch64-unknown-linux-gnu)"
            echo "  windows     Windows x86_64 (x86_64-pc-windows-msvc)"
            echo ""
            echo "If no targets are specified, all 5 are built."
            echo ""
            echo "Options:"
            echo "  --skip-binaries  Skip downloading sidecar binaries (adb/rclone/7z)"
            echo "  --verbose, -v    Show detailed build output"
            echo "  --help, -h       Show this help"
            exit 0
            ;;
        *)
            if is_valid_target "$arg"; then
                REQUESTED+=("$arg")
            else
                echo "ERROR: Unknown target '$arg'"
                echo "Valid targets: ${ALL_ALIASES[*]}"
                exit 1
            fi
            ;;
    esac
done

if [ ${#REQUESTED[@]} -eq 0 ]; then
    REQUESTED=("${ALL_ALIASES[@]}")
fi

mkdir -p "$DIST_DIR"

# ── Helpers ──────────────────────────────────────────────────────────

log()  { echo "==> $*"; }
step() { echo "  -> $*"; }
fail() { echo "ERROR: $*" >&2; exit 1; }

check_rust_target() {
    local triple="$1"
    # Check if rustup is available
    if command -v rustup &>/dev/null; then
        if ! rustup target list --installed | grep -q "^${triple}$"; then
            echo ""
            echo "Rust target '$triple' is not installed."
            echo "Install it with:  rustup target add $triple"
            return 1
        fi
    else
        # No rustup - check if this is the host target
        local host_triple
        host_triple=$(rustc -vV | grep host | cut -d' ' -f2)
        if [ "$triple" != "$host_triple" ]; then
            echo ""
            echo "Rust is installed without rustup (likely via Homebrew)."
            echo "Cross-compilation to '$triple' requires rustup."
            echo "Install rustup from https://rustup.rs/"
            return 1
        fi
    fi
}

check_prerequisites() {
    local target_alias="$1"
    local triple
    triple=$(get_target_triple "$target_alias")

    # Check Rust target is installed
    check_rust_target "$triple" || return 1

    # Platform-specific checks
    case "$target_alias" in
        linux)
            if [[ "$(uname -s)" == "Darwin" ]]; then
                # Cross-compiling from macOS to Linux x86_64
                if ! command -v x86_64-unknown-linux-gnu-gcc &>/dev/null && \
                   ! command -v cross &>/dev/null; then
                    echo ""
                    echo "Cross-compiling to Linux x86_64 from macOS requires either:"
                    echo "  1. A cross-linker: brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu"
                    echo "  2. Or 'cross': cargo install cross"
                    return 1
                fi
            fi
            ;;
        linux-arm)
            if [[ "$(uname -s)" == "Darwin" ]]; then
                # Cross-compiling from macOS to Linux ARM
                if ! command -v aarch64-unknown-linux-gnu-gcc &>/dev/null && \
                   ! command -v cross &>/dev/null; then
                    echo ""
                    echo "Cross-compiling to Linux ARM from macOS requires either:"
                    echo "  1. A cross-linker: brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu"
                    echo "  2. Or 'cross': cargo install cross"
                    return 1
                fi
            fi
            ;;
        windows)
            if [[ "$(uname -s)" != MINGW* && "$(uname -s)" != MSYS* && "$(uname -s)" != CYGWIN* ]]; then
                # Cross-compiling to Windows
                if ! command -v cargo-xwin &>/dev/null && ! command -v cross &>/dev/null; then
                    echo ""
                    echo "Cross-compiling to Windows requires either:"
                    echo "  1. cargo-xwin: cargo install cargo-xwin"
                    echo "  2. Or 'cross': cargo install cross"
                    return 1
                fi
            fi
            ;;
    esac
    return 0
}

# ── Build frontend once ──────────────────────────────────────────────

log "Building frontend..."
npm run build --prefix "$PROJECT_DIR/frontend"

# ── Build each target ────────────────────────────────────────────────

SUCCEEDED=()
FAILED=()
SKIPPED=()

for alias in "${REQUESTED[@]}"; do
    triple=$(get_target_triple "$alias")

    log "[$alias] Target: $triple"

    # Check prerequisites
    if ! check_prerequisites "$alias"; then
        step "SKIPPED (missing prerequisites)"
        SKIPPED+=("$alias")
        continue
    fi

    # Download sidecar binaries
    if [ "$SKIP_BINARIES" = false ]; then
        step "Downloading sidecar binaries..."
        if ! "$DOWNLOAD_SCRIPT" "$triple"; then
            step "FAILED to download binaries"
            FAILED+=("$alias")
            continue
        fi
    else
        step "Skipping binary download (--skip-binaries)"
        # Ensure placeholder stubs exist so tauri build doesn't fail
        EXE=""
        [[ "$triple" == *windows* ]] && EXE=".exe"
        for bin in adb rclone 7z; do
            stub="$PROJECT_DIR/src-tauri/binaries/${bin}-${triple}${EXE}"
            if [ ! -f "$stub" ]; then
                touch "$stub"
                chmod +x "$stub"
            fi
        done
    fi

    # Build with Tauri
    step "Building Tauri app..."
    BUILD_ARGS=(--target "$triple")
    if [ "$VERBOSE" = true ]; then
        BUILD_ARGS+=(--verbose)
    fi

    # Use TAURI_SKIP_DEVSERVER_CHECK to avoid dev server issues during cross-build
    # Pass --no-bundle for cross-compilation targets that can't produce native bundles,
    # but attempt full bundle for native targets
    export TAURI_SKIP_DEVSERVER_CHECK=true

    if (cd "$PROJECT_DIR" && npx @tauri-apps/cli build "${BUILD_ARGS[@]}"); then
        step "Build succeeded!"
        SUCCEEDED+=("$alias")

        # Copy artifacts to dist/
        step "Collecting artifacts..."
        ARTIFACT_DIR="$DIST_DIR/$alias"
        # Clean existing artifacts for this target to avoid stale bundles (like the old Rookie Desktop.app)
        rm -rf "$ARTIFACT_DIR"
        mkdir -p "$ARTIFACT_DIR"

        BUNDLE_DIR="$PROJECT_DIR/src-tauri/target/${triple}/release/bundle"
        BIN_DIR="$PROJECT_DIR/src-tauri/target/${triple}/release"

        case "$alias" in
            mac-arm|mac-x86)
                # .dmg or .app
                if ls "$BUNDLE_DIR"/dmg/*.dmg 2>/dev/null; then
                    cp "$BUNDLE_DIR"/dmg/*.dmg "$ARTIFACT_DIR/"
                fi
                if ls "$BUNDLE_DIR"/macos/*.app 2>/dev/null; then
                    cp -R "$BUNDLE_DIR"/macos/*.app "$ARTIFACT_DIR/"
                fi
                ;;
            linux|linux-arm)
                # .deb, .AppImage, or raw binary
                if ls "$BUNDLE_DIR"/deb/*.deb 2>/dev/null; then
                    cp "$BUNDLE_DIR"/deb/*.deb "$ARTIFACT_DIR/"
                fi
                if ls "$BUNDLE_DIR"/appimage/*.AppImage 2>/dev/null; then
                    cp "$BUNDLE_DIR"/appimage/*.AppImage "$ARTIFACT_DIR/"
                fi
                if [ -f "$BIN_DIR/veteran-desktop" ]; then
                    cp "$BIN_DIR/veteran-desktop" "$ARTIFACT_DIR/"
                fi
                ;;
            windows)
                # .msi or .exe (NSIS)
                if ls "$BUNDLE_DIR"/msi/*.msi 2>/dev/null; then
                    cp "$BUNDLE_DIR"/msi/*.msi "$ARTIFACT_DIR/"
                fi
                if ls "$BUNDLE_DIR"/nsis/*.exe 2>/dev/null; then
                    cp "$BUNDLE_DIR"/nsis/*.exe "$ARTIFACT_DIR/"
                fi
                if [ -f "$BIN_DIR/veteran-desktop.exe" ]; then
                    cp "$BIN_DIR/veteran-desktop.exe" "$ARTIFACT_DIR/"
                fi
                ;;
        esac

        step "Artifacts -> $ARTIFACT_DIR"
    else
        step "Build FAILED"
        FAILED+=("$alias")
    fi
done

# ── Summary ──────────────────────────────────────────────────────────

echo ""
echo "========================================="
echo "  Build Summary"
echo "========================================="
if [ ${#SUCCEEDED[@]} -gt 0 ]; then
    echo "  Succeeded: ${SUCCEEDED[*]}"
fi
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "  Failed:    ${FAILED[*]}"
fi
if [ ${#SKIPPED[@]} -gt 0 ]; then
    echo "  Skipped:   ${SKIPPED[*]}"
fi
echo ""
echo "  Artifacts: $DIST_DIR/"
if [ ${#SUCCEEDED[@]} -gt 0 ]; then
    ls -d "$DIST_DIR"/*/ 2>/dev/null | while read -r d; do
        echo "    $(basename "$d")/"
        ls "$d" 2>/dev/null | sed 's/^/      /'
    done
fi
echo "========================================="

# Exit with error if any builds failed
[ ${#FAILED[@]} -eq 0 ]
