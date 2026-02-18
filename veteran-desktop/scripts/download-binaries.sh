#!/usr/bin/env bash
set -euo pipefail

# Download adb, rclone, and 7z binaries for Tauri sidecar bundling.
# Usage: ./scripts/download-binaries.sh [target-triple]
# Example: ./scripts/download-binaries.sh aarch64-apple-darwin

TARGET="${1:-}"
BINDIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/binaries"
mkdir -p "$BINDIR"

# Check for --force flag
FORCE=false
if [ "${2:-}" = "--force" ]; then
    FORCE=true
fi

if [ -z "$TARGET" ]; then
    # Auto-detect
    ARCH="$(uname -m)"
    OS="$(uname -s)"
    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                x86_64)        TARGET="x86_64-apple-darwin" ;;
                *) echo "Unsupported arch: $ARCH"; exit 1 ;;
            esac ;;
        Linux)
            case "$ARCH" in
                x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
                aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported arch: $ARCH"; exit 1 ;;
            esac ;;
        *) echo "Unsupported OS: $OS. For Windows, run this in WSL or adapt the script."; exit 1 ;;
    esac
    echo "Auto-detected target: $TARGET"
fi

mkdir -p "$BINDIR"

EXE=""
case "$TARGET" in
    *windows*) EXE=".exe" ;;
esac

# Check if binaries already exist
ADB_BIN="$BINDIR/adb-${TARGET}${EXE}"
RCLONE_BIN="$BINDIR/rclone-${TARGET}${EXE}"
SEVENZ_BIN="$BINDIR/7z-${TARGET}${EXE}"

if [ "$FORCE" = false ] && [ -f "$ADB_BIN" ] && [ -f "$RCLONE_BIN" ] && [ -f "$SEVENZ_BIN" ]; then
    echo "==> Binaries for $TARGET already exist (use --force to re-download):"
    ls -lh "$ADB_BIN" "$RCLONE_BIN" "$SEVENZ_BIN"
    exit 0
fi

echo "==> Downloading binaries for $TARGET into $BINDIR"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# --- ADB ---
echo "--- adb ---"
case "$TARGET" in
    *apple-darwin*)
        ADB_URL="https://dl.google.com/android/repository/platform-tools-latest-darwin.zip" ;;
    *linux*)
        ADB_URL="https://dl.google.com/android/repository/platform-tools-latest-linux.zip" ;;
    *windows*)
        ADB_URL="https://dl.google.com/android/repository/platform-tools-latest-windows.zip" ;;
    *) echo "No adb download for $TARGET"; exit 1 ;;
esac

curl -fSL "$ADB_URL" -o "$TMPDIR/platform-tools.zip"
unzip -q -o "$TMPDIR/platform-tools.zip" -d "$TMPDIR/adb-extract"
cp "$TMPDIR/adb-extract/platform-tools/adb${EXE}" "$BINDIR/adb-${TARGET}${EXE}"
chmod +x "$BINDIR/adb-${TARGET}${EXE}"
echo "  -> $BINDIR/adb-${TARGET}${EXE}"

# --- rclone ---
echo "--- rclone ---"
RCLONE_VERSION="current"
case "$TARGET" in
    aarch64-apple-darwin)
        RCLONE_PLATFORM="osx-arm64" ;;
    x86_64-apple-darwin)
        RCLONE_PLATFORM="osx-amd64" ;;
    x86_64-unknown-linux-gnu)
        RCLONE_PLATFORM="linux-amd64" ;;
    aarch64-unknown-linux-gnu)
        RCLONE_PLATFORM="linux-arm64" ;;
    x86_64-pc-windows-msvc)
        RCLONE_PLATFORM="windows-amd64" ;;
    *) echo "No rclone download for $TARGET"; exit 1 ;;
esac

RCLONE_URL="https://downloads.rclone.org/rclone-${RCLONE_VERSION}-${RCLONE_PLATFORM}.zip"
curl -fSL "$RCLONE_URL" -o "$TMPDIR/rclone.zip"
unzip -q -o "$TMPDIR/rclone.zip" -d "$TMPDIR/rclone-extract"
# rclone zip contains a directory like rclone-vX.Y.Z-platform/
RCLONE_BIN=$(find "$TMPDIR/rclone-extract" -name "rclone${EXE}" -type f | head -1)
cp "$RCLONE_BIN" "$BINDIR/rclone-${TARGET}${EXE}"
chmod +x "$BINDIR/rclone-${TARGET}${EXE}"
echo "  -> $BINDIR/rclone-${TARGET}${EXE}"

# --- 7z (7zz standalone) ---
echo "--- 7z ---"
case "$TARGET" in
    aarch64-apple-darwin)
        SEVENZ_URL="https://www.7-zip.org/a/7z2408-mac.tar.xz"
        SEVENZ_FORMAT="tar.xz"
        SEVENZ_BIN_NAME="7zz" ;;
    x86_64-apple-darwin)
        SEVENZ_URL="https://www.7-zip.org/a/7z2408-mac.tar.xz"
        SEVENZ_FORMAT="tar.xz"
        SEVENZ_BIN_NAME="7zz" ;;
    x86_64-unknown-linux-gnu)
        SEVENZ_URL="https://www.7-zip.org/a/7z2408-linux-x64.tar.xz"
        SEVENZ_FORMAT="tar.xz"
        SEVENZ_BIN_NAME="7zz" ;;
    aarch64-unknown-linux-gnu)
        SEVENZ_URL="https://www.7-zip.org/a/7z2408-linux-arm64.tar.xz"
        SEVENZ_FORMAT="tar.xz"
        SEVENZ_BIN_NAME="7zz" ;;
    x86_64-pc-windows-msvc)
        SEVENZ_URL="https://www.7-zip.org/a/7z2408-extra.7z"
        SEVENZ_FORMAT="7z"
        SEVENZ_BIN_NAME="7za.exe" ;;
    *) echo "No 7z download for $TARGET"; exit 1 ;;
esac

mkdir -p "$TMPDIR/7z-extract"
if [ "$SEVENZ_FORMAT" = "tar.xz" ]; then
    curl -fSL "$SEVENZ_URL" -o "$TMPDIR/7z.tar.xz"
    tar -xf "$TMPDIR/7z.tar.xz" -C "$TMPDIR/7z-extract"
    SEVENZ_BIN=$(find "$TMPDIR/7z-extract" -name "$SEVENZ_BIN_NAME" -type f | head -1)
elif [ "$SEVENZ_FORMAT" = "7z" ]; then
    curl -fSL "$SEVENZ_URL" -o "$TMPDIR/7z-extra.7z"
    # Need an existing 7z to extract; fall back to system 7z
    if command -v 7z &>/dev/null; then
        7z x "$TMPDIR/7z-extra.7z" -o"$TMPDIR/7z-extract" -y
    elif command -v 7zz &>/dev/null; then
        7zz x "$TMPDIR/7z-extra.7z" -o"$TMPDIR/7z-extract" -y
    else
        echo "ERROR: Need 7z or 7zz on PATH to extract Windows 7z archive"
        exit 1
    fi
    SEVENZ_BIN=$(find "$TMPDIR/7z-extract" -name "$SEVENZ_BIN_NAME" -type f | head -1)
fi

if [ -z "${SEVENZ_BIN:-}" ]; then
    echo "ERROR: Could not find $SEVENZ_BIN_NAME in downloaded archive"
    exit 1
fi

cp "$SEVENZ_BIN" "$BINDIR/7z-${TARGET}${EXE}"
chmod +x "$BINDIR/7z-${TARGET}${EXE}"
echo "  -> $BINDIR/7z-${TARGET}${EXE}"

echo ""
echo "==> Done! Binaries in $BINDIR:"
ls -lh "$BINDIR"/*-"${TARGET}"* 2>/dev/null || echo "(none found)"
