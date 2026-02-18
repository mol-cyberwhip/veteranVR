#!/usr/bin/env node
import { chromium } from "playwright";
import { fileURLToPath } from "url";
import path from "path";
import os from "os";
import fs from "fs";
import { execSync } from "child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR = path.resolve(__dirname, "..", "example_images");
const APP_URL = "http://localhost:1420";
const THUMBNAILS_DIR = path.join(os.homedir(), ".veteran", "cache", "thumbnails");

// ─── Mock Data (real package names with thumbnails) ────────────────────────────

const MOCK_GAMES = [
  { package_name: "com.beatgames.beatsaber", game_name: "Beat Saber", release_name: "Beat Saber v2059", version_code: 2059, version_name: "1.37.0", size: "5.1 GB", last_updated: "2025-12-31", is_favorite: true, is_new: false, popularity_rank: 1, downloads: "98940", is_downloaded: false },
  { package_name: "com.camouflaj.manta", game_name: "Batman: Arkham Shadow", release_name: "Batman Arkham Shadow v350961", version_code: 350961, version_name: "1.0.0", size: "29 GB", last_updated: "2025-10-15", is_favorite: false, is_new: true, popularity_rank: 2, downloads: "98330", is_downloaded: false },
  { package_name: "com.twistedpixelgames.PILO", game_name: "Marvel's Deadpool VR", release_name: "Deadpool VR v8742", version_code: 8742, version_name: "1.0.0", size: "34.3 GB", last_updated: "2025-12-13", is_favorite: false, is_new: true, popularity_rank: 3, downloads: "98710", is_downloaded: false },
  { package_name: "com.Sanzaru.Wrath2", game_name: "Asgard's Wrath 2", release_name: "Asgards Wrath 2 v5513", version_code: 5513, version_name: "1.0.0", size: "33.5 GB", last_updated: "2025-05-02", is_favorite: true, is_new: false, popularity_rank: 4, downloads: "97050", is_downloaded: false },
  { package_name: "com.vertigogames.azs2", game_name: "Arizona Sunshine 2", release_name: "Arizona Sunshine 2 v112988", version_code: 112988, version_name: "1.0.0", size: "23.9 GB", last_updated: "2025-10-17", is_favorite: false, is_new: false, popularity_rank: 5, downloads: "97620", is_downloaded: false },
  { package_name: "com.SDI.BHM", game_name: "Skydance's BEHEMOTH", release_name: "BEHEMOTH v55911500", version_code: 55911500, version_name: "1.0.0", size: "28.2 GB", last_updated: "2025-06-14", is_favorite: false, is_new: false, popularity_rank: 6, downloads: "94210", is_downloaded: false },
  { package_name: "com.xrgames.hitman3reloaded", game_name: "HITMAN 3 VR: Reloaded", release_name: "HITMAN 3 VR v3469", version_code: 3469, version_name: "1.0.0", size: "28.0 GB", last_updated: "2025-05-20", is_favorite: false, is_new: false, popularity_rank: 7, downloads: "96930", is_downloaded: false },
  { package_name: "com.RealitiesIO.puzzlingPlaces", game_name: "Puzzling Places", release_name: "Puzzling Places v772", version_code: 772, version_name: "1.0.0", size: "56.0 GB", last_updated: "2026-01-10", is_favorite: false, is_new: false, popularity_rank: 8, downloads: "91860", is_downloaded: false },
  { package_name: "com.vertigogames.azs1hd", game_name: "Arizona Sunshine Remake", release_name: "AZ Sunshine Remake v112989", version_code: 112989, version_name: "1.0.0", size: "23.3 GB", last_updated: "2025-10-19", is_favorite: true, is_new: false, popularity_rank: 9, downloads: "96590", is_downloaded: false },
  { package_name: "games.b4t.epicrollercoasters.oculus", game_name: "Epic Roller Coasters", release_name: "Epic Roller Coasters v6199", version_code: 6199, version_name: "1.0.0", size: "40.1 GB", last_updated: "2025-12-06", is_favorite: false, is_new: false, popularity_rank: 10, downloads: "96140", is_downloaded: false },
  { package_name: "com.Virtuos.R6DLM", game_name: "Medal of Honor: Above and Beyond", release_name: "Medal of Honor v12297", version_code: 12297, version_name: "1.0.0", size: "43.9 GB", last_updated: "2025-05-24", is_favorite: false, is_new: false, popularity_rank: 11, downloads: "93600", is_downloaded: false },
  { package_name: "com.PineStudio.EscapeSimulator", game_name: "Escape Simulator", release_name: "Escape Simulator v36357", version_code: 36357, version_name: "1.0.0", size: "25.8 GB", last_updated: "2025-10-02", is_favorite: false, is_new: false, popularity_rank: 12, downloads: "81340", is_downloaded: false },
];

const MOCK_DEVICE_STATE = {
  status: "connected",
  status_message: "Meta Quest 3 (2JECW1234567)",
  troubleshooting: "Device connected and ready",
  can_download: true,
  can_install: true,
  download_only_mode: false,
  selected_serial: "2JECW1234567",
  selection_source: "auto",
  devices: [
    { serial: "2JECW1234567", state: "device", model: "Quest 3", product: "eureka", is_selected: true, is_connected: true }
  ],
  storage: { total_mb: 131072, used_mb: 52428, free_mb: 78644 },
  battery: { level_percent: 87, status: "Discharging", is_charging: false, temperature_c: 28.5 },
  wireless: { saved_endpoint: null, auto_reconnect_enabled: false, last_attempt_at: null, last_endpoint: null, last_status: null, last_error: null },
  keep_awake: { enabled: false, interval_seconds: 30, active_count: 0, active_operation_ids: [] }
};

const MOCK_CATALOG_STATUS = {
  synced: true,
  source: "cache",
  game_count: 2509,
  has_config: true,
  config_base_uri: "https://example.com",
  sync_error: null,
  sync_in_progress: false,
  thumbnails_dir: THUMBNAILS_DIR,
  notes_dir: path.join(os.homedir(), ".veteran", "cache", "notes"),
  cache_dir: path.join(os.homedir(), ".veteran", "cache"),
  cache_age_hours: 1.5,
  cache_stale: false,
  message: "2509 games synced (cache: 1.5h old)"
};

const MOCK_DOWNLOAD_QUEUE = {
  queue: [
    { package_name: "com.beatgames.beatsaber", release_name: "Beat Saber v2059", game_name: "Beat Saber", status: "downloading", progress_percent: 67.3, speed: "42.1 MB/s", eta: "1m 12s", bytes_transferred: 3456000000, total_bytes: 5100000000 },
    { package_name: "com.SDI.BHM", release_name: "BEHEMOTH v55911500", game_name: "Skydance's BEHEMOTH", status: "queued", progress_percent: 0, speed: "", eta: "", bytes_transferred: 0, total_bytes: 0 },
    { package_name: "com.xrgames.hitman3reloaded", release_name: "HITMAN 3 VR v3469", game_name: "HITMAN 3 VR: Reloaded", status: "queued", progress_percent: 0, speed: "", eta: "", bytes_transferred: 0, total_bytes: 0 },
    { package_name: "com.RealitiesIO.puzzlingPlaces", release_name: "Puzzling Places v772", game_name: "Puzzling Places", status: "completed", progress_percent: 100, speed: "", eta: "", bytes_transferred: 56000000000, total_bytes: 56000000000 },
    { package_name: "com.PineStudio.EscapeSimulator", release_name: "Escape Simulator v36357", game_name: "Escape Simulator", status: "failed", progress_percent: 34.2, speed: "", eta: "", bytes_transferred: 8820000000, total_bytes: 25800000000 },
  ],
  active_download: { package_name: "com.beatgames.beatsaber", release_name: "Beat Saber v2059", game_name: "Beat Saber", status: "downloading", progress_percent: 67.3, speed: "42.1 MB/s", eta: "1m 12s", bytes_transferred: 3456000000, total_bytes: 5100000000 },
  queued_count: 2,
  total_count: 5,
  processing: true,
};

const MOCK_INSTALLED_APPS = {
  apps: [
    { package_name: "com.beatgames.beatsaber", app_name: "Beat Saber", version_code: "1900", version_name: "1.35.0", size: "2.1 GB", in_catalog: true, update_available: true, is_system_app: false, install_time: null, last_update_time: null, game_name: "Beat Saber", catalog_version_code: "2059", installed_version_code: "1900" },
    { package_name: "com.camouflaj.manta", app_name: "Batman: Arkham Shadow", version_code: "340000", version_name: "0.9.0", size: "28.5 GB", in_catalog: true, update_available: true, is_system_app: false, install_time: null, last_update_time: null, game_name: "Batman: Arkham Shadow", catalog_version_code: "350961", installed_version_code: "340000" },
    { package_name: "com.Sanzaru.Wrath2", app_name: "Asgard's Wrath 2", version_code: "5513", version_name: "1.0.0", size: "33.5 GB", in_catalog: true, update_available: false, is_system_app: false, install_time: null, last_update_time: null, game_name: "Asgard's Wrath 2", catalog_version_code: "5513", installed_version_code: "5513" },
    { package_name: "com.vertigogames.azs2", app_name: "Arizona Sunshine 2", version_code: "112988", version_name: "1.0.0", size: "23.9 GB", in_catalog: true, update_available: false, is_system_app: false, install_time: null, last_update_time: null, game_name: "Arizona Sunshine 2", catalog_version_code: "112988", installed_version_code: "112988" },
    { package_name: "com.RealitiesIO.puzzlingPlaces", app_name: "Puzzling Places", version_code: "772", version_name: "1.0.0", size: "56.0 GB", in_catalog: true, update_available: false, is_system_app: false, install_time: null, last_update_time: null, game_name: "Puzzling Places", catalog_version_code: "772", installed_version_code: "772" },
    { package_name: "com.oculus.browser", app_name: "Meta Quest Browser", version_code: "80", version_name: "33.2", size: "150 MB", in_catalog: false, update_available: false, is_system_app: false, install_time: null, last_update_time: null, game_name: null, catalog_version_code: null, installed_version_code: null },
    { package_name: "com.oculus.vrshell.home", app_name: "Meta Quest Home", version_code: "45", version_name: "67.0", size: "200 MB", in_catalog: false, update_available: false, is_system_app: true, install_time: null, last_update_time: null, game_name: null, catalog_version_code: null, installed_version_code: null },
  ],
  count: 7,
  has_updates: true,
};

const MOCK_BACKUPS = [
  { package_name: "com.beatgames.beatsaber", backup_path: "/backups/com.beatgames.beatsaber.tar.gz", size_bytes: 52428800, created_at: Date.now() / 1000 - 3600 },
  { package_name: "com.Sanzaru.Wrath2", backup_path: "/backups/com.Sanzaru.Wrath2.tar.gz", size_bytes: 104857600, created_at: Date.now() / 1000 - 86400 },
  { package_name: "com.RealitiesIO.puzzlingPlaces", backup_path: "/backups/com.RealitiesIO.puzzlingPlaces.tar.gz", size_bytes: 31457280, created_at: Date.now() / 1000 - 172800 },
];

const MOCK_EVENTS = { events: [] };

// ─── Build thumbnail lookup ────────────────────────────────────────────────────

// Map package_name -> absolute path to thumbnail jpg
const thumbnailMap = {};
for (const game of MOCK_GAMES) {
  const thumbPath = path.join(THUMBNAILS_DIR, `${game.package_name}.jpg`);
  if (fs.existsSync(thumbPath)) {
    thumbnailMap[game.package_name] = thumbPath;
  }
}
// Also add installed apps that might not be in MOCK_GAMES
for (const app of MOCK_INSTALLED_APPS.apps) {
  if (!thumbnailMap[app.package_name]) {
    const thumbPath = path.join(THUMBNAILS_DIR, `${app.package_name}.jpg`);
    if (fs.existsSync(thumbPath)) {
      thumbnailMap[app.package_name] = thumbPath;
    }
  }
}

console.log(`Found ${Object.keys(thumbnailMap).length} thumbnails for mock games`);

// ─── Tauri Bridge Mock ─────────────────────────────────────────────────────────

function getTauriBridgeScript() {
  // We serve thumbnails via a custom URL scheme that we intercept with Playwright route
  const THUMB_BASE = "http://localhost:1420/__thumb__/";

  return `
    const THUMB_BASE = ${JSON.stringify(THUMB_BASE)};
    const THUMBNAIL_MAP = ${JSON.stringify(thumbnailMap)};

    const mockInvoke = async (cmd, args) => {
      const MOCK_DEVICE_STATE = ${JSON.stringify(MOCK_DEVICE_STATE)};
      const MOCK_CATALOG_STATUS = ${JSON.stringify(MOCK_CATALOG_STATUS)};
      const MOCK_DOWNLOAD_QUEUE = ${JSON.stringify(MOCK_DOWNLOAD_QUEUE)};
      const MOCK_INSTALLED_APPS = ${JSON.stringify(MOCK_INSTALLED_APPS)};
      const MOCK_GAMES = ${JSON.stringify(MOCK_GAMES)};
      const MOCK_BACKUPS = ${JSON.stringify(MOCK_BACKUPS)};
      const MOCK_EVENTS = ${JSON.stringify(MOCK_EVENTS)};

      switch (cmd) {
        case 'backend_ready_state':
          return { ready: true, pid: 12345, version: '0.1.1' };
        case 'backend_device_state':
          return MOCK_DEVICE_STATE;
        case 'backend_catalog_status':
          return MOCK_CATALOG_STATUS;
        case 'backend_catalog_sync':
          return MOCK_CATALOG_STATUS;
        case 'backend_catalog_load_cache':
          return MOCK_CATALOG_STATUS;
        case 'backend_catalog_library':
          const query = (args?.query || '').toLowerCase();
          const games = query
            ? MOCK_GAMES.filter(g => g.game_name.toLowerCase().includes(query))
            : MOCK_GAMES;
          return {
            games: games,
            total: games.length,
            offset: args?.offset || 0,
            limit: args?.limit || 500,
            query: args?.query || '',
            sort_by: args?.sortBy || 'popularity',
            sort_ascending: args?.sortAscending ?? true,
            filter: args?.filter || 'all',
            favorites_count: 3,
          };
        case 'backend_catalog_thumbnail_path': {
          const pkg = args?.packageName;
          const thumbPath = THUMBNAIL_MAP[pkg];
          if (thumbPath) {
            return { thumbnail_exists: true, thumbnail_path: thumbPath };
          }
          return { thumbnail_exists: false, thumbnail_path: null };
        }
        case 'backend_catalog_note':
          return { note: null };
        case 'backend_download_queue_status':
          return MOCK_DOWNLOAD_QUEUE;
        case 'backend_installed_apps':
          return MOCK_INSTALLED_APPS;
        case 'backend_list_backups':
          return MOCK_BACKUPS;
        case 'poll_backend_events':
          return MOCK_EVENTS;
        case 'backend_recover':
          return { recovered: true, message: 'Backend recovered' };
        case 'search_youtube_trailer':
          return 'Q6KRU1SocGg';
        case 'backend_install_status':
          return { history: [], history_count: 0 };
        case 'frontend_log':
          return null;
        default:
          console.log('[Mock] Unhandled invoke:', cmd, args);
          return {};
      }
    };

    // @tauri-apps/api/core uses window.__TAURI_INTERNALS__.invoke
    window.__TAURI_INTERNALS__ = {
      invoke: mockInvoke,
      convertFileSrc: (filePath, protocol) => {
        // Convert file path to our intercepted thumbnail URL
        return THUMB_BASE + encodeURIComponent(filePath);
      },
      transformCallback: (cb) => {
        const id = Math.random().toString(36).slice(2);
        window['_' + id] = cb;
        return id;
      },
      metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
    };

    // Also set __TAURI__ for code that checks window.__TAURI__
    window.__TAURI__ = {
      core: {
        invoke: mockInvoke,
        convertFileSrc: (filePath) => {
          return THUMB_BASE + encodeURIComponent(filePath);
        },
      },
      event: {
        listen: () => Promise.resolve(() => {}),
        emit: () => Promise.resolve(),
      },
    };
  `;
}

// ─── Screenshot Logic ──────────────────────────────────────────────────────────

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
  });

  const page = await context.newPage();

  // Intercept thumbnail requests and serve local files
  await page.route("**/\__thumb__/**", async (route) => {
    const url = route.request().url();
    const encodedPath = url.split("/__thumb__/")[1];
    if (!encodedPath) {
      return route.abort();
    }
    const filePath = decodeURIComponent(encodedPath);
    try {
      const body = fs.readFileSync(filePath);
      await route.fulfill({
        status: 200,
        contentType: "image/jpeg",
        body,
      });
    } catch {
      await route.abort();
    }
  });

  // Inject Tauri mock before any page script runs
  await page.addInitScript(getTauriBridgeScript());

  // Navigate to the app
  await page.goto(APP_URL, { waitUntil: "networkidle" });

  // Wait for the app to render
  await page.waitForSelector(".app-shell", { timeout: 10000 });
  await page.waitForTimeout(3000); // Let polling settle and thumbnails load

  // ─── Start video recording ───────────────────────────────────────────────
  const videoPath = path.join(OUTPUT_DIR, "demo.webm");
  await page.video()?.path(); // ensure video dir is ready
  // We record via context-level video; re-create context with video
  await browser.close();

  const browser2 = await chromium.launch({ headless: true });
  const context2 = await browser2.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
    recordVideo: { dir: OUTPUT_DIR, size: { width: 1440, height: 900 } },
  });
  const page2 = await context2.newPage();

  // Re-setup route and mocks
  await page2.route("**/\__thumb__/**", async (route) => {
    const url = route.request().url();
    const encodedPath = url.split("/__thumb__/")[1];
    if (!encodedPath) return route.abort();
    const filePath = decodeURIComponent(encodedPath);
    try {
      const body = fs.readFileSync(filePath);
      await route.fulfill({ status: 200, contentType: "image/jpeg", body });
    } catch { await route.abort(); }
  });
  await page2.addInitScript(getTauriBridgeScript());

  await page2.goto(APP_URL, { waitUntil: "networkidle" });
  await page2.waitForSelector(".app-shell", { timeout: 10000 });
  await page2.waitForTimeout(3000);

  // ─── 1. Library View ─────────────────────────────────────────────────────
  console.log("📸 Taking Library screenshot...");
  await page2.click('.sidebar-nav-item:has-text("Library")');
  await page2.waitForTimeout(2500);
  await page2.screenshot({ path: path.join(OUTPUT_DIR, "library.png"), fullPage: false });

  // ─── 2. Game Detail Modal (YouTube) ──────────────────────────────────────
  console.log("📸 Taking Game Detail Modal screenshot...");
  const cardTitle2 = await page2.$(".game-card .card-meta-top");
  if (cardTitle2) {
    await cardTitle2.click();
    try {
      await page2.waitForSelector(".modal-overlay", { timeout: 5000 });
      await page2.waitForTimeout(4000);
      await page2.screenshot({ path: path.join(OUTPUT_DIR, "game_details.png"), fullPage: false });
      await page2.click(".modal-close-btn");
      await page2.waitForTimeout(800);
    } catch (e) {
      console.log("  Fallback: triggering modal via JS...");
      await page2.evaluate(() => { document.querySelector('.game-card')?.click(); });
      await page2.waitForTimeout(1000);
      try {
        await page2.waitForSelector(".modal-overlay", { timeout: 3000 });
        await page2.waitForTimeout(4000);
        await page2.screenshot({ path: path.join(OUTPUT_DIR, "game_details.png"), fullPage: false });
        await page2.click(".modal-close-btn");
        await page2.waitForTimeout(800);
      } catch (e2) {
        console.warn("⚠️  Could not open game detail modal - skipping screenshot");
      }
    }
  }

  // ─── 3. Downloads View ───────────────────────────────────────────────────
  console.log("📸 Taking Downloads screenshot...");
  await page2.click('.sidebar-nav-item:has-text("Downloads")');
  await page2.waitForTimeout(2000);
  await page2.screenshot({ path: path.join(OUTPUT_DIR, "downloads.png"), fullPage: false });

  // ─── 4. Installed View ───────────────────────────────────────────────────
  console.log("📸 Taking Installed screenshot...");
  await page2.click('.sidebar-nav-item:has-text("Installed")');
  await page2.waitForTimeout(2500);
  await page2.screenshot({ path: path.join(OUTPUT_DIR, "installed.png"), fullPage: false });

  // ─── 5. Backups View ────────────────────────────────────────────────────
  console.log("📸 Taking Backups screenshot...");
  await page2.click('.sidebar-nav-item:has-text("Backups")');
  await page2.waitForTimeout(2000);
  await page2.screenshot({ path: path.join(OUTPUT_DIR, "backups.png"), fullPage: false });

  // ─── 6. Diagnostics View ────────────────────────────────────────────────
  console.log("📸 Taking Diagnostics screenshot...");
  await page2.click('.sidebar-nav-item:has-text("Diagnostics")');
  await page2.waitForTimeout(2000);
  await page2.screenshot({ path: path.join(OUTPUT_DIR, "diagnostics.png"), fullPage: false });

  // ─── Scroll back to library for a nice ending ────────────────────────────
  await page2.click('.sidebar-nav-item:has-text("Library")');
  await page2.waitForTimeout(1500);

  // ─── Stop recording and convert to GIF ───────────────────────────────────
  const recordedVideoPath = await page2.video()?.path();
  await page2.close(); // finalize video
  await context2.close();
  await browser2.close();

  if (recordedVideoPath) {
    const gifPath = path.join(OUTPUT_DIR, "demo.gif");
    console.log("\n🎬 Converting video to GIF...");
    try {
      execSync(
        `ffmpeg -y -i "${recordedVideoPath}" -vf "fps=12,scale=720:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" -loop 0 "${gifPath}"`,
        { stdio: "pipe" }
      );
      // Clean up the webm
      fs.unlinkSync(recordedVideoPath);
      console.log(`✅ GIF saved to ${gifPath}`);
    } catch (err) {
      console.error("⚠️  GIF conversion failed:", err.message);
      // Rename the webm instead
      const webmDest = path.join(OUTPUT_DIR, "demo.webm");
      fs.renameSync(recordedVideoPath, webmDest);
      console.log(`📹 Video saved as ${webmDest} (GIF conversion failed)`);
    }
  }

  console.log(`\n✅ All screenshots saved to ${OUTPUT_DIR}`);
}

main().catch((err) => {
  console.error("Screenshot failed:", err);
  process.exit(1);
});
