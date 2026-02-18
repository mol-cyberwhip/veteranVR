# Veteran Quest App

Android APK for Quest 2/3 to browse catalog and install/uninstall content directly from headset.

## Current Status

This initial scaffold sets up:
- Kotlin + Jetpack Compose app shell
- Pure Kotlin core modules with unit tests (`core-model`, `core-catalog`, `core-installer`)
- Search/sort/filter UX and uninstall toggles in app UI
- CI workflow for unit tests + debug APK artifact

The real downloader/extractor/installer backend is intentionally the next step.

## Stack

- Kotlin + Gradle Kotlin DSL
- Jetpack Compose (panel-mode UI)
- Core logic as pure Kotlin modules for JVM unit tests
- `minSdk=29`, `targetSdk=29` for Quest-oriented storage/install behavior

## Modules

- `core-model`: shared models and query definitions
- `core-catalog`: config decoding, catalog parsing, hash + search/sort/filter logic
- `core-installer`: `install.txt` parser and install plan generation
- `app`: Android UI + repository wiring

## Build

```bash
cd veteran-quest-app
./gradlew :core-model:test :core-catalog:test :core-installer:test
./gradlew :app:assembleDebug
```

## Why Release Signing Is Extra Work

Release signing itself is not conceptually hard, but production-ready signing needs:
- Keystore generation and secure storage strategy
- CI secret handling for keystore + passwords
- Signed release workflow/versioning policy
- Install/update key continuity guarantees

That is why the first CI output is debug APK, then release signing can be added once key-management policy is decided.

## Next Implementation Slice

1. Real remote sync pipeline (config fetch, `meta.7z` download, passworded extract, thumbnail/note cache)
2. Background download queue with pause/resume in-process
3. Android `PackageInstaller` flow + OBB copy + uninstall toggles
4. Minimal diagnostics/log panel
