// @ts-nocheck
import type {
  CatalogStatus,
  CatalogSearchResult,
  LibraryResult,
  FavoritesToggleResult,
  DeviceState,
  DownloadQueueStatus,
  OperationStatus,
} from "@bindings";

export const IPC_SCHEMA_VERSION = 1;
const OPERATION_STATES = new Set([
  "queued",
  "running",
  "cancelling",
  "succeeded",
  "failed",
  "cancelled",
]);
const DEVICE_STATUSES = new Set([
  "connected",
  "multiple_connected",
  "selection_required",
  "unauthorized",
  "offline",
  "no_device",
  "adb_unavailable",
]);
const DEVICE_SELECTION_SOURCES = new Set(["none", "auto", "manual"]);

function isObject(value: any): value is Record<string, any> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function isTerminalState(state: any): boolean {
  return state === "succeeded" || state === "failed" || state === "cancelled";
}

export function assertOperationSnapshot(snapshot: any): OperationStatus {
  if (!isObject(snapshot)) {
    throw new Error("Operation snapshot must be a JSON object.");
  }

  if (typeof snapshot.operation_id !== "string" || !snapshot.operation_id) {
    throw new Error("operation.operation_id must be a non-empty string.");
  }

  if (typeof snapshot.state !== "string" || !snapshot.state) {
    throw new Error("operation.state must be a non-empty string.");
  }
  if (!OPERATION_STATES.has(snapshot.state)) {
    throw new Error(`operation.state ${String(snapshot.state)} is not supported.`);
  }

  if (!Number.isInteger(snapshot.state_version) || snapshot.state_version < 0) {
    throw new Error("operation.state_version must be a non-negative integer.");
  }

  if (!Array.isArray(snapshot.state_history) || snapshot.state_history.length === 0) {
    throw new Error("operation.state_history must be a non-empty array.");
  }
  let previousVersion = -1;
  for (const entry of snapshot.state_history) {
    if (!isObject(entry)) {
      throw new Error("operation.state_history entries must be objects.");
    }
    if (!Number.isInteger(entry.version) || entry.version < 0) {
      throw new Error("operation.state_history[].version must be a non-negative integer.");
    }
    if (entry.version <= previousVersion) {
      throw new Error("operation.state_history[].version must be strictly increasing.");
    }
    previousVersion = entry.version;
    if (typeof entry.state !== "string" || !OPERATION_STATES.has(entry.state)) {
      throw new Error("operation.state_history[].state must be a supported state string.");
    }
    if (typeof entry.entered_at !== "number") {
      throw new Error("operation.state_history[].entered_at must be numeric.");
    }
    if (typeof entry.reason !== "string" || !entry.reason) {
      throw new Error("operation.state_history[].reason must be a non-empty string.");
    }
  }
  const latestState = snapshot.state_history[snapshot.state_history.length - 1];
  if (latestState.state !== snapshot.state) {
    throw new Error("operation.state must match the latest state_history entry.");
  }
  if (latestState.version !== snapshot.state_version) {
    throw new Error("operation.state_version must match the latest state_history version.");
  }

  if (!isObject(snapshot.progress)) {
    throw new Error("operation.progress must be an object.");
  }

  const progress = snapshot.progress;
  if (typeof progress.percent !== "number") {
    throw new Error("operation.progress.percent must be numeric.");
  }
  if (typeof progress.completed_steps !== "number") {
    throw new Error("operation.progress.completed_steps must be numeric.");
  }
  if (typeof progress.total_steps !== "number") {
    throw new Error("operation.progress.total_steps must be numeric.");
  }

  if (typeof snapshot.cancel_requested !== "boolean") {
    throw new Error("operation.cancel_requested must be a boolean.");
  }
  if (
    snapshot.cancel_requested_at !== null &&
    typeof snapshot.cancel_requested_at !== "number"
  ) {
    throw new Error("operation.cancel_requested_at must be numeric or null.");
  }
  if (snapshot.cancel_requested && snapshot.cancel_requested_at === null) {
    throw new Error("operation.cancel_requested_at must be set when cancel_requested is true.");
  }

  if (typeof snapshot.terminal !== "boolean") {
    throw new Error("operation.terminal must be a boolean.");
  }
  if (snapshot.terminal_at !== null && typeof snapshot.terminal_at !== "number") {
    throw new Error("operation.terminal_at must be numeric or null.");
  }
  if (snapshot.terminal && snapshot.terminal_at === null) {
    throw new Error("operation.terminal_at must be set for terminal operations.");
  }

  if (snapshot.keep_awake !== undefined && snapshot.keep_awake !== null) {
    if (!isObject(snapshot.keep_awake)) {
      throw new Error("operation.keep_awake must be an object when provided.");
    }
    if (typeof snapshot.keep_awake.enabled !== "boolean") {
      throw new Error("operation.keep_awake.enabled must be a boolean.");
    }
    if (snapshot.keep_awake.enabled) {
      if (
        !Number.isInteger(snapshot.keep_awake.interval_seconds) ||
        snapshot.keep_awake.interval_seconds < 1
      ) {
        throw new Error(
          "operation.keep_awake.interval_seconds must be a positive integer when enabled."
        );
      }
    } else if (
      snapshot.keep_awake.interval_seconds !== null &&
      snapshot.keep_awake.interval_seconds !== undefined &&
      !Number.isInteger(snapshot.keep_awake.interval_seconds)
    ) {
      throw new Error("operation.keep_awake.interval_seconds must be an integer or null.");
    }
    if (
      !Number.isInteger(snapshot.keep_awake.ticks_sent) ||
      snapshot.keep_awake.ticks_sent < 0
    ) {
      throw new Error("operation.keep_awake.ticks_sent must be a non-negative integer.");
    }
    if (
      snapshot.keep_awake.last_sent_at !== null &&
      snapshot.keep_awake.last_sent_at !== undefined &&
      typeof snapshot.keep_awake.last_sent_at !== "number"
    ) {
      throw new Error("operation.keep_awake.last_sent_at must be numeric or null.");
    }
  }

  return snapshot;
}

export function assertResponseEnvelope(payload, { expectedRequestId } = {}) {
  if (!isObject(payload)) {
    throw new Error("Response envelope must be a JSON object.");
  }

  if (payload.schema_version !== IPC_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported schema_version: ${String(payload.schema_version)}; expected ${IPC_SCHEMA_VERSION}.`
    );
  }

  if (payload.kind !== "response") {
    throw new Error("Response envelope kind must be 'response'.");
  }

  if (expectedRequestId !== undefined && payload.id !== expectedRequestId) {
    throw new Error(
      `Response id mismatch: expected ${String(expectedRequestId)}, got ${String(payload.id)}.`
    );
  }

  if (payload.status === "ok") {
    if (!isObject(payload.result)) {
      throw new Error("Success response requires object result payload.");
    }
    return payload;
  }

  if (payload.status === "error") {
    if (!isObject(payload.error)) {
      throw new Error("Error response requires error object.");
    }
    if (typeof payload.error.code !== "string" || !payload.error.code) {
      throw new Error("Error response requires non-empty error.code.");
    }
    if (typeof payload.error.message !== "string" || !payload.error.message) {
      throw new Error("Error response requires non-empty error.message.");
    }
    return payload;
  }

  throw new Error("Response envelope status must be 'ok' or 'error'.");
}

export function assertEventEnvelope(payload: any): any {
  if (!isObject(payload)) {
    throw new Error("Event envelope must be a JSON object.");
  }

  if (payload.schema_version !== IPC_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported schema_version: ${String(payload.schema_version)}; expected ${IPC_SCHEMA_VERSION}.`
    );
  }

  if (payload.kind !== "event") {
    throw new Error("Event envelope kind must be 'event'.");
  }

  if (typeof payload.event !== "string" || !payload.event) {
    throw new Error("Event envelope requires a non-empty event name.");
  }

  if (typeof payload.timestamp !== "number") {
    throw new Error("Event envelope requires numeric timestamp.");
  }

  assertOperationSnapshot(payload.operation);
  return payload;
}

export function assertDeviceStateSnapshot(payload: any): DeviceState {
  if (!isObject(payload)) {
    throw new Error("Device state payload must be a JSON object.");
  }

  if (typeof payload.status !== "string" || !DEVICE_STATUSES.has(payload.status)) {
    throw new Error("device.status must be a supported status string.");
  }
  if (typeof payload.status_message !== "string" || !payload.status_message) {
    throw new Error("device.status_message must be a non-empty string.");
  }
  if (typeof payload.troubleshooting !== "string" || !payload.troubleshooting) {
    throw new Error("device.troubleshooting must be a non-empty string.");
  }
  if (typeof payload.can_download !== "boolean") {
    throw new Error("device.can_download must be a boolean.");
  }
  if (typeof payload.can_install !== "boolean") {
    throw new Error("device.can_install must be a boolean.");
  }
  if (typeof payload.download_only_mode !== "boolean") {
    throw new Error("device.download_only_mode must be a boolean.");
  }
  if (payload.selected_serial !== null && typeof payload.selected_serial !== "string") {
    throw new Error("device.selected_serial must be a string or null.");
  }
  if (
    typeof payload.selection_source !== "string" ||
    !DEVICE_SELECTION_SOURCES.has(payload.selection_source)
  ) {
    throw new Error("device.selection_source must be one of none/auto/manual.");
  }
  if (!Array.isArray(payload.devices)) {
    throw new Error("device.devices must be an array.");
  }

  for (const device of payload.devices) {
    if (!isObject(device)) {
      throw new Error("device.devices entries must be objects.");
    }
    if (typeof device.serial !== "string" || !device.serial) {
      throw new Error("device.devices[].serial must be a non-empty string.");
    }
    if (typeof device.state !== "string" || !device.state) {
      throw new Error("device.devices[].state must be a non-empty string.");
    }
    if (typeof device.model !== "string") {
      throw new Error("device.devices[].model must be a string.");
    }
    if (typeof device.product !== "string") {
      throw new Error("device.devices[].product must be a string.");
    }
    if (typeof device.is_selected !== "boolean") {
      throw new Error("device.devices[].is_selected must be a boolean.");
    }
    if (typeof device.is_connected !== "boolean") {
      throw new Error("device.devices[].is_connected must be a boolean.");
    }
  }

  if (payload.storage !== null && payload.storage !== undefined) {
    if (!isObject(payload.storage)) {
      throw new Error("device.storage must be an object or null.");
    }
    for (const key of ["total_mb", "used_mb", "free_mb"]) {
      if (!Number.isInteger(payload.storage[key]) || payload.storage[key] < 0) {
        throw new Error(`device.storage.${key} must be a non-negative integer.`);
      }
    }
  }

  if (payload.battery !== null && payload.battery !== undefined) {
    if (!isObject(payload.battery)) {
      throw new Error("device.battery must be an object or null.");
    }
    if (
      payload.battery.level_percent !== null &&
      payload.battery.level_percent !== undefined &&
      (!Number.isInteger(payload.battery.level_percent) || payload.battery.level_percent < 0)
    ) {
      throw new Error("device.battery.level_percent must be a non-negative integer or null.");
    }
    if (typeof payload.battery.status !== "string" || !payload.battery.status) {
      throw new Error("device.battery.status must be a non-empty string.");
    }
    if (typeof payload.battery.is_charging !== "boolean") {
      throw new Error("device.battery.is_charging must be a boolean.");
    }
    if (
      payload.battery.temperature_c !== null &&
      payload.battery.temperature_c !== undefined &&
      typeof payload.battery.temperature_c !== "number"
    ) {
      throw new Error("device.battery.temperature_c must be numeric or null.");
    }
  }

  if (!isObject(payload.wireless)) {
    throw new Error("device.wireless must be an object.");
  }
  if (
    payload.wireless.saved_endpoint !== null &&
    payload.wireless.saved_endpoint !== undefined &&
    typeof payload.wireless.saved_endpoint !== "string"
  ) {
    throw new Error("device.wireless.saved_endpoint must be a string or null.");
  }
  if (typeof payload.wireless.auto_reconnect_enabled !== "boolean") {
    throw new Error("device.wireless.auto_reconnect_enabled must be a boolean.");
  }
  if (
    payload.wireless.last_attempt_at !== null &&
    payload.wireless.last_attempt_at !== undefined &&
    typeof payload.wireless.last_attempt_at !== "number"
  ) {
    throw new Error("device.wireless.last_attempt_at must be numeric or null.");
  }
  for (const key of ["last_endpoint", "last_status", "last_error"]) {
    if (
      payload.wireless[key] !== null &&
      payload.wireless[key] !== undefined &&
      typeof payload.wireless[key] !== "string"
    ) {
      throw new Error(`device.wireless.${key} must be a string or null.`);
    }
  }

  if (!isObject(payload.keep_awake)) {
    throw new Error("device.keep_awake must be an object.");
  }
  if (typeof payload.keep_awake.enabled !== "boolean") {
    throw new Error("device.keep_awake.enabled must be a boolean.");
  }
  if (
    !Number.isInteger(payload.keep_awake.interval_seconds) ||
    payload.keep_awake.interval_seconds < 1
  ) {
    throw new Error("device.keep_awake.interval_seconds must be a positive integer.");
  }
  if (
    !Number.isInteger(payload.keep_awake.active_count) ||
    payload.keep_awake.active_count < 0
  ) {
    throw new Error("device.keep_awake.active_count must be a non-negative integer.");
  }
  if (!Array.isArray(payload.keep_awake.active_operation_ids)) {
    throw new Error("device.keep_awake.active_operation_ids must be an array.");
  }
  for (const operationId of payload.keep_awake.active_operation_ids) {
    if (typeof operationId !== "string" || !operationId) {
      throw new Error("device.keep_awake.active_operation_ids[] must be non-empty strings.");
    }
  }
  if (payload.keep_awake.active_operation_ids.length !== payload.keep_awake.active_count) {
    throw new Error(
      "device.keep_awake.active_count must match active_operation_ids length."
    );
  }

  return payload;
}

export function summarizeOperationSnapshot(snapshot: any, fallbackOperationId: string | null = null): any {
  const operation = assertOperationSnapshot(snapshot);
  const operationId = operation.operation_id || fallbackOperationId;
  const state = operation.state;
  const operationName =
    typeof operation.operation === "string" && operation.operation
      ? operation.operation
      : "unknown";
  const percent = operation.progress.percent;
  const statusText = `Operation ${operationId} (${operationName}) is ${state} at ${percent.toFixed(1)}%.`;

  return {
    operationId,
    state,
    operationName,
    percent,
    statusText,
    terminal: isTerminalState(state),
  };
}

const CATALOG_SOURCES = new Set(["none", "network", "cache"]);

export function assertCatalogStatusSnapshot(payload: any): CatalogStatus {
  if (!isObject(payload)) {
    throw new Error("Catalog status payload must be a JSON object.");
  }
  if (typeof payload.synced !== "boolean") {
    throw new Error("catalog.synced must be a boolean.");
  }
  if (typeof payload.source !== "string" || !CATALOG_SOURCES.has(payload.source)) {
    throw new Error("catalog.source must be one of none/network/cache.");
  }
  if (!Number.isInteger(payload.game_count) || payload.game_count < 0) {
    throw new Error("catalog.game_count must be a non-negative integer.");
  }
  if (typeof payload.has_config !== "boolean") {
    throw new Error("catalog.has_config must be a boolean.");
  }
  if (
    payload.config_base_uri !== null &&
    payload.config_base_uri !== undefined &&
    typeof payload.config_base_uri !== "string"
  ) {
    throw new Error("catalog.config_base_uri must be a string or null.");
  }
  if (
    payload.sync_error !== null &&
    payload.sync_error !== undefined &&
    typeof payload.sync_error !== "string"
  ) {
    throw new Error("catalog.sync_error must be a string or null.");
  }
  if (typeof payload.thumbnails_dir !== "string") {
    throw new Error("catalog.thumbnails_dir must be a string.");
  }
  if (typeof payload.notes_dir !== "string") {
    throw new Error("catalog.notes_dir must be a string.");
  }
  if (typeof payload.cache_dir !== "string") {
    throw new Error("catalog.cache_dir must be a string.");
  }
  if (
    payload.cache_age_hours !== null &&
    payload.cache_age_hours !== undefined &&
    typeof payload.cache_age_hours !== "number"
  ) {
    throw new Error("catalog.cache_age_hours must be a number or null.");
  }
  if (
    payload.cache_stale !== null &&
    payload.cache_stale !== undefined &&
    typeof payload.cache_stale !== "boolean"
  ) {
    throw new Error("catalog.cache_stale must be a boolean or null.");
  }
  return payload;
}

export function assertCatalogSearchResult(payload: any): CatalogSearchResult {
  if (!isObject(payload)) {
    throw new Error("Catalog search payload must be a JSON object.");
  }
  if (!Array.isArray(payload.games)) {
    throw new Error("catalog_search.games must be an array.");
  }
  if (!Number.isInteger(payload.total) || payload.total < 0) {
    throw new Error("catalog_search.total must be a non-negative integer.");
  }
  if (!Number.isInteger(payload.offset) || payload.offset < 0) {
    throw new Error("catalog_search.offset must be a non-negative integer.");
  }
  if (!Number.isInteger(payload.limit) || payload.limit < 1) {
    throw new Error("catalog_search.limit must be a positive integer.");
  }
  if (typeof payload.query !== "string") {
    throw new Error("catalog_search.query must be a string.");
  }
  return payload;
}

const LIBRARY_SORT_COLUMNS = new Set(["name", "popularity", "date", "size"]);
const LIBRARY_FILTERS = new Set(["all", "favorites", "new", "popular"]);

export function assertLibraryResult(payload: any): LibraryResult {
  if (!isObject(payload)) {
    throw new Error("Library payload must be a JSON object.");
  }
  if (!Array.isArray(payload.games)) {
    throw new Error("library.games must be an array.");
  }
  for (const game of payload.games) {
    if (!isObject(game)) {
      throw new Error("library.games[] must be objects.");
    }
    if (typeof game.is_favorite !== "boolean") {
      throw new Error("library.games[].is_favorite must be a boolean.");
    }
    if (typeof game.is_new !== "boolean") {
      throw new Error("library.games[].is_new must be a boolean.");
    }
    if (!Number.isInteger(game.popularity_rank)) {
      throw new Error("library.games[].popularity_rank must be an integer.");
    }
  }
  if (!Number.isInteger(payload.total) || payload.total < 0) {
    throw new Error("library.total must be a non-negative integer.");
  }
  if (!Number.isInteger(payload.offset) || payload.offset < 0) {
    throw new Error("library.offset must be a non-negative integer.");
  }
  if (!Number.isInteger(payload.limit) || payload.limit < 1) {
    throw new Error("library.limit must be a positive integer.");
  }
  if (typeof payload.query !== "string") {
    throw new Error("library.query must be a string.");
  }
  if (typeof payload.sort_by !== "string" || !LIBRARY_SORT_COLUMNS.has(payload.sort_by)) {
    throw new Error("library.sort_by must be one of name/popularity/date/size.");
  }
  if (typeof payload.sort_ascending !== "boolean") {
    throw new Error("library.sort_ascending must be a boolean.");
  }
  if (typeof payload.filter !== "string" || !LIBRARY_FILTERS.has(payload.filter)) {
    throw new Error("library.filter must be one of all/favorites/new/popular.");
  }
  if (!Number.isInteger(payload.favorites_count) || payload.favorites_count < 0) {
    throw new Error("library.favorites_count must be a non-negative integer.");
  }
  return payload;
}

export function assertFavoritesToggleResult(payload: any): FavoritesToggleResult {
  if (!isObject(payload)) {
    throw new Error("Favorites toggle payload must be a JSON object.");
  }
  if (typeof payload.package_name !== "string" || !payload.package_name) {
    throw new Error("favorites_toggle.package_name must be a non-empty string.");
  }
  if (typeof payload.is_favorite !== "boolean") {
    throw new Error("favorites_toggle.is_favorite must be a boolean.");
  }
  if (!Array.isArray(payload.favorites)) {
    throw new Error("favorites_toggle.favorites must be an array.");
  }
  return payload;
}

const DOWNLOAD_QUEUE_ITEM_STATES = new Set([
  "queued", "downloading", "completed", "failed", "cancelled", "retrying"
]);

export function assertDownloadQueueItem(item) {
  if (!isObject(item)) {
    throw new Error("Download queue item must be a JSON object.");
  }
  if (typeof item.package_name !== "string" || !item.package_name) {
    throw new Error("download_queue_item.package_name must be a non-empty string.");
  }
  if (typeof item.status !== "string" || !DOWNLOAD_QUEUE_ITEM_STATES.has(item.status)) {
    throw new Error("download_queue_item.status must be a valid state.");
  }
  if (typeof item.progress_percent !== "number") {
    throw new Error("download_queue_item.progress_percent must be numeric.");
  }
  if (typeof item.speed !== "string") {
    throw new Error("download_queue_item.speed must be a string.");
  }
  if (typeof item.eta !== "string") {
    throw new Error("download_queue_item.eta must be a string.");
  }
  return item;
}

export function assertDownloadQueueStatus(payload: any): DownloadQueueStatus {
  if (!isObject(payload)) {
    throw new Error("Download queue status payload must be a JSON object.");
  }
  if (!Array.isArray(payload.queue)) {
    throw new Error("download_queue.queue must be an array.");
  }
  for (const item of payload.queue) {
    assertDownloadQueueItem(item);
  }
  if (!Number.isInteger(payload.queued_count) || payload.queued_count < 0) {
    throw new Error("download_queue.queued_count must be a non-negative integer.");
  }
  if (!Number.isInteger(payload.total_count) || payload.total_count < 0) {
    throw new Error("download_queue.total_count must be a non-negative integer.");
  }
  if (typeof payload.processing !== "boolean") {
    throw new Error("download_queue.processing must be a boolean.");
  }
  return payload;
}

export function assertDownloadLocation(payload) {
  if (!isObject(payload)) {
    throw new Error("Download location payload must be a JSON object.");
  }
  if (typeof payload.download_dir !== "string" || !payload.download_dir) {
    throw new Error("download_location.download_dir must be a non-empty string.");
  }
  if (typeof payload.exists !== "boolean") {
    throw new Error("download_location.exists must be a boolean.");
  }
  if (typeof payload.free_bytes !== "number") {
    throw new Error("download_location.free_bytes must be numeric.");
  }
  if (typeof payload.file_count !== "number") {
    throw new Error("download_location.file_count must be numeric.");
  }
  return payload;
}

export function assertCheckLocalResult(payload) {
  if (!isObject(payload)) {
    throw new Error("Check local payload must be a JSON object.");
  }
  if (typeof payload.package_name !== "string" || !payload.package_name) {
    throw new Error("check_local.package_name must be a non-empty string.");
  }
  if (typeof payload.has_local_files !== "boolean") {
    throw new Error("check_local.has_local_files must be a boolean.");
  }
  if (typeof payload.local_size_bytes !== "number") {
    throw new Error("check_local.local_size_bytes must be numeric.");
  }
  return payload;
}

export function assertDeleteDownloadResult(payload) {
  if (!isObject(payload)) {
    throw new Error("Delete download payload must be a JSON object.");
  }
  if (typeof payload.package_name !== "string" || !payload.package_name) {
    throw new Error("delete_download.package_name must be a non-empty string.");
  }
  if (typeof payload.deleted !== "boolean") {
    throw new Error("delete_download.deleted must be a boolean.");
  }
  if (typeof payload.freed_bytes !== "number") {
    throw new Error("delete_download.freed_bytes must be numeric.");
  }
  return payload;
}

// ---- Install contract helpers ----

const INSTALL_STATUS_VALUES = new Set([
  "pending", "extracting", "installing_apk", "pushing_obb",
  "running_install_commands", "verifying", "completed", "failed", "cancelled",
]);

const INSTALL_TIMELINE_STEPS = new Set([
  "extracting", "checking_install_txt", "installing_apk", "pushing_obb",
  "running_install_commands", "verifying", "completed",
]);

const INSTALL_TIMELINE_STATUSES = new Set(["running", "completed", "failed", "cancelled"]);

export function assertInstallGameResult(payload) {
  if (!isObject(payload)) throw new Error("Install game payload must be a JSON object.");
  if (typeof payload.operation_id !== "string" || !payload.operation_id) {
    throw new Error("install_game.operation_id must be a non-empty string.");
  }
  if (typeof payload.package_name !== "string" || !payload.package_name) {
    throw new Error("install_game.package_name must be a non-empty string.");
  }
  return payload;
}

export function assertInstallStatusResult(payload) {
  if (!isObject(payload)) throw new Error("Install status payload must be a JSON object.");
  if (!Array.isArray(payload.history)) throw new Error("install_status.history must be a list.");
  if (typeof payload.history_count !== "number") throw new Error("install_status.history_count must be a number.");
  return payload;
}

export function formatEventLine(eventPayload: any): string {
  const envelope = assertEventEnvelope(eventPayload);
  const operation = envelope.operation;
  const progressPercent = operation.progress?.percent;

  let line = `[${envelope.timestamp}] ${envelope.event} ${operation.operation_id} state=${operation.state}`;
  if (typeof progressPercent === "number") {
    line += ` progress=${progressPercent.toFixed(1)}%`;
  }
  if (envelope.error && typeof envelope.error.message === "string") {
    line += ` error=${envelope.error.message}`;
  }
  return line;
}
