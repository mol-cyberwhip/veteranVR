# Veteran Quest App

Veteran Quest is a native Android APK for Quest 2/3 that mirrors Veteran Desktop's core sideload flow directly on headset.

## v1 User Story

`Catalog -> Browse -> Install + Uninstall`

v1 includes:
- Real catalog sync from `vrp-public.json` + `meta.7z`
- 4-hour cache policy parity with desktop behavior
- Thumbnail-backed browse, search, sort, and default non-mod filter
- Single-worker download queue with pause/resume while app is alive
- Passworded 7z extraction with bundled `7zz` asset
- Install pipeline: APK + OBB + `install.txt` allowlist mode (warn/skip unsupported commands)
- Uninstall with `Keep OBB` and `Keep Data` toggles
- In-app operation diagnostics log panel

Out of scope for v1:
- Update action UX
- Resume after process death/reboot
- Full desktop-grade diagnostics suite

## Project Layout

- `core-model`: shared domain models
- `core-catalog`: catalog/config parsing + hash/chunk planning
- `core-installer`: install.txt parsing/planning
- `app`: Android platform services + Compose UI

## Build and Checks

```bash
cd veteran-quest-app
./gradlew :core-model:test :core-catalog:test :core-installer:test
./gradlew :app:testDebugUnitTest
./gradlew detekt
./gradlew :app:assembleDebug
```

## Runtime Notes

- App expects `app/src/main/assets/7zz` to be executable on Quest arm64.
- Catalog/download requests use `User-Agent: rclone/v1.73.0` to match remote compatibility expectations.
- Install actions are gated by:
  - Unknown app install permission
  - All files access
  - Minimum free storage threshold

## Manual Runbook (Quest 2 + Quest 3)

1. Fresh install app, grant setup permissions from gate.
2. Sync catalog, verify thumbnails load.
3. Verify search/sort/non-mod filter behavior.
4. Install APK-only title.
5. Install APK+OBB title.
6. Install title with `install.txt` containing supported and unsupported commands.
7. Pause/resume download while app is backgrounded but alive.
8. Uninstall with both toggle combinations.
9. Verify downloaded artifacts are cleaned after successful install.
10. Verify cache skip (<4h) and force sync behavior.
