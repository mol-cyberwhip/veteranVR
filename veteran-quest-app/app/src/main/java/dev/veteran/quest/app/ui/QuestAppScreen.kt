package dev.veteran.quest.app.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.service.PermissionGateService
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.SortBy
import java.io.File
import java.util.Locale

private val Navy = Color(0xFF070E1A)
private val NavySoft = Color(0xFF10213A)
private val Mint = Color(0xFF32D8A0)
private val Cyan = Color(0xFF4DB9FF)
private val Amber = Color(0xFFF5BC52)
private val Danger = Color(0xFFFF7272)
private val Border = Color(0x2A9EC9F7)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuestAppScreen(
    viewModel: QuestViewModel,
    permissionGateService: PermissionGateService,
) {
    val state by viewModel.state.collectAsState()
    val context = LocalContext.current
    val view = LocalView.current

    DisposableEffect(state.activeOperation, state.keepAwakeDuringOps) {
        val keepAwake = state.keepAwakeDuringOps && state.activeOperation != null
        view.keepScreenOn = keepAwake
        onDispose {
            view.keepScreenOn = false
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                brush = Brush.linearGradient(
                    listOf(
                        Color(0xFF03070F),
                        Color(0xFF0A1628),
                        Color(0xFF0F1A2E),
                    ),
                ),
            ),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(14.dp),
        ) {
            HeaderStrip(
                message = state.message,
                activeOperation = state.activeOperation,
                onPause = { op -> viewModel.pauseOperation(op.operationId) },
                onResume = { op -> viewModel.resumeOperation(op.operationId) },
            )

            Spacer(modifier = Modifier.height(10.dp))

            AnimatedVisibility(
                visible = state.permissionStatus?.isReady == false,
                enter = fadeIn(),
                exit = fadeOut(),
            ) {
                val status = state.permissionStatus
                if (status != null) {
                    SetupGate(
                        statusText = "Setup required for installs",
                        hasInstall = status.canInstallPackages,
                        hasFiles = status.hasAllFilesAccess,
                        freeBytes = status.freeBytes,
                        minBytes = status.minRequiredBytes,
                        onOpenInstallSettings = {
                            context.startActivity(permissionGateService.openInstallPermissionSettingsIntent())
                        },
                        onOpenFilesSettings = {
                            context.startActivity(permissionGateService.openAllFilesPermissionSettingsIntent())
                        },
                        onRefresh = viewModel::refreshPermissionStatus,
                    )
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            Row(
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                ControlsPane(
                    modifier = Modifier.width(360.dp).fillMaxHeight(),
                    search = state.search,
                    sortBy = state.sortBy,
                    sortAscending = state.sortAscending,
                    filter = state.filter,
                    syncing = state.syncing,
                    keepObb = state.keepObbOnUninstall,
                    keepData = state.keepDataOnUninstall,
                    keepAwake = state.keepAwakeDuringOps,
                    logs = state.logs,
                    onSearch = viewModel::onSearchChanged,
                    onSort = viewModel::onSortByChanged,
                    onToggleSort = viewModel::onSortDirectionToggled,
                    onFilter = viewModel::onFilterChanged,
                    onSync = { viewModel.refreshCatalog(force = true) },
                    onKeepObb = viewModel::onKeepObbChanged,
                    onKeepData = viewModel::onKeepDataChanged,
                    onKeepAwake = viewModel::onKeepAwakeChanged,
                )

                CatalogPane(
                    modifier = Modifier.weight(1f).fillMaxHeight(),
                    items = state.games,
                    operations = state.operations,
                    onInstall = { viewModel.enqueueInstall(it.game) },
                    onUninstall = { viewModel.uninstall(it.game) },
                )
            }
        }
    }
}

@Composable
private fun HeaderStrip(
    message: String?,
    activeOperation: DownloadOperation?,
    onPause: (DownloadOperation) -> Unit,
    onResume: (DownloadOperation) -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(Color(0x88111F35))
            .border(width = 1.dp, color = Border, shape = RoundedCornerShape(16.dp))
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column {
            Text(
                text = "Veteran Quest",
                style = MaterialTheme.typography.headlineSmall.copy(
                    fontWeight = FontWeight.Bold,
                    fontFamily = FontFamily.SansSerif,
                ),
                color = Color(0xFFE7F6FF),
            )
            Text(
                text = message ?: "Ready",
                style = MaterialTheme.typography.bodyMedium,
                color = Color(0xFFB7D8EE),
            )
        }

        if (activeOperation != null) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "${activeOperation.releaseName}: ${activeOperation.state.name.lowercase()} ${activeOperation.progressPercent.toInt()}%",
                    color = Cyan,
                    style = MaterialTheme.typography.labelLarge.copy(fontFamily = FontFamily.Monospace),
                )
                if (activeOperation.state == DownloadState.DOWNLOADING) {
                    OutlinedButton(onClick = { onPause(activeOperation) }) { Text("Pause") }
                }
                if (activeOperation.state == DownloadState.PAUSED) {
                    Button(onClick = { onResume(activeOperation) }) { Text("Resume") }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ControlsPane(
    modifier: Modifier,
    search: String,
    sortBy: SortBy,
    sortAscending: Boolean,
    filter: LibraryFilter,
    syncing: Boolean,
    keepObb: Boolean,
    keepData: Boolean,
    keepAwake: Boolean,
    logs: List<OperationLogEntry>,
    onSearch: (String) -> Unit,
    onSort: (SortBy) -> Unit,
    onToggleSort: () -> Unit,
    onFilter: (LibraryFilter) -> Unit,
    onSync: () -> Unit,
    onKeepObb: (Boolean) -> Unit,
    onKeepData: (Boolean) -> Unit,
    onKeepAwake: (Boolean) -> Unit,
) {
    var logLevel by remember { mutableStateOf("ALL") }
    var opFilter by remember { mutableStateOf("") }

    Card(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = Color(0x88111F35)),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            OutlinedTextField(
                value = search,
                onValueChange = onSearch,
                label = { Text("Search") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                listOf(SortBy.POPULARITY, SortBy.NAME, SortBy.DATE, SortBy.SIZE).forEach { sort ->
                    FilterChip(
                        selected = sortBy == sort,
                        onClick = { onSort(sort) },
                        label = { Text(sort.name.lowercase().replaceFirstChar { it.uppercase() }) },
                    )
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                listOf(LibraryFilter.NON_MODS, LibraryFilter.ALL, LibraryFilter.NEW, LibraryFilter.POPULAR).forEach { chip ->
                    FilterChip(
                        selected = filter == chip,
                        onClick = { onFilter(chip) },
                        label = { Text(chip.name.lowercase().replace('_', '-').replaceFirstChar { it.uppercase() }) },
                    )
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Button(onClick = onSync) {
                    Text(if (syncing) "Syncing..." else "Sync Catalog")
                }
                OutlinedButton(onClick = onToggleSort) {
                    Text(if (sortAscending) "Asc" else "Desc")
                }
            }

            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(checked = keepObb, onCheckedChange = onKeepObb)
                Text("Keep OBB", color = Color(0xFFE4F1FB))
                Spacer(modifier = Modifier.width(8.dp))
                Checkbox(checked = keepData, onCheckedChange = onKeepData)
                Text("Keep Data", color = Color(0xFFE4F1FB))
                Spacer(modifier = Modifier.width(8.dp))
                Checkbox(checked = keepAwake, onCheckedChange = onKeepAwake)
                Text("Keep Awake", color = Color(0xFFE4F1FB))
            }

            Text(
                text = "Operation Log",
                color = Color(0xFFE7F6FF),
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.SemiBold),
            )

            OutlinedTextField(
                value = opFilter,
                onValueChange = { opFilter = it },
                singleLine = true,
                label = { Text("Filter by operation id") },
                modifier = Modifier.fillMaxWidth(),
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("ALL", "INFO", "WARN", "ERROR").forEach { level ->
                    FilterChip(
                        selected = level == logLevel,
                        onClick = { logLevel = level },
                        label = { Text(level) },
                    )
                }
            }

            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Navy)
                    .border(1.dp, Border, RoundedCornerShape(12.dp)),
                contentPadding = PaddingValues(8.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                val filtered = logs.filter { entry ->
                    (logLevel == "ALL" || entry.level == logLevel) &&
                        (opFilter.isBlank() || entry.operationId.contains(opFilter, ignoreCase = true))
                }

                items(filtered.take(200), key = { "${it.timestampMs}-${it.message}" }) { entry ->
                    Text(
                        text = "[${entry.level}] ${entry.stage} ${entry.message}",
                        style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
                        color = when (entry.level) {
                            "ERROR" -> Danger
                            "WARN" -> Amber
                            else -> Color(0xFF9EE0FF)
                        },
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

@Composable
private fun CatalogPane(
    modifier: Modifier,
    items: List<LibraryItemUi>,
    operations: List<DownloadOperation>,
    onInstall: (LibraryItemUi) -> Unit,
    onUninstall: (LibraryItemUi) -> Unit,
) {
    Card(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = Color(0x66122541)),
    ) {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = 220.dp),
            modifier = Modifier
                .fillMaxSize()
                .padding(10.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(bottom = 24.dp),
        ) {
            items(items, key = { "${it.game.packageName}-${it.game.releaseName}" }) { item ->
                val op = operations.firstOrNull { it.packageName == item.game.packageName }
                GameCard(
                    item = item,
                    operation = op,
                    onInstall = { onInstall(item) },
                    onUninstall = { onUninstall(item) },
                )
            }
        }
    }
}

@Composable
private fun GameCard(
    item: LibraryItemUi,
    operation: DownloadOperation?,
    onInstall: () -> Unit,
    onUninstall: () -> Unit,
) {
    Card(
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = NavySoft),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, Border, RoundedCornerShape(16.dp)),
    ) {
        Column(modifier = Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            val thumbModel = if (item.thumbnailExists) File(item.thumbnailPath) else null
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(110.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Navy),
                contentAlignment = Alignment.Center,
            ) {
                if (thumbModel != null) {
                    AsyncImage(
                        model = thumbModel,
                        contentDescription = item.game.gameName,
                        modifier = Modifier.fillMaxSize(),
                    )
                } else {
                    Text("No Thumb", color = Color(0xFF85A5BD))
                }
            }

            Text(
                text = item.game.gameName,
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
                color = Color(0xFFE7F6FF),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = item.game.releaseName,
                style = MaterialTheme.typography.labelSmall,
                color = Color(0xFFA9C9E0),
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = item.game.size,
                    color = Cyan,
                    style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
                )
                if (item.isInstalled) {
                    StatusPill("Installed", Mint)
                }
                if (item.game.isModded) {
                    StatusPill("Mod", Amber)
                }
                if (operation != null) {
                    val color = when (operation.state) {
                        DownloadState.FAILED -> Danger
                        DownloadState.PAUSED -> Amber
                        DownloadState.COMPLETED -> Mint
                        else -> Cyan
                    }
                    StatusPill(operation.state.name.lowercase(), color)
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onInstall) {
                    Text("Install")
                }
                OutlinedButton(onClick = onUninstall) {
                    Text("Uninstall")
                }
            }
        }
    }
}

@Composable
private fun SetupGate(
    statusText: String,
    hasInstall: Boolean,
    hasFiles: Boolean,
    freeBytes: Long,
    minBytes: Long,
    onOpenInstallSettings: () -> Unit,
    onOpenFilesSettings: () -> Unit,
    onRefresh: () -> Unit,
) {
    Card(
        shape = RoundedCornerShape(14.dp),
        colors = CardDefaults.cardColors(containerColor = Color(0x99A8671E)),
        modifier = Modifier.fillMaxWidth(),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(statusText, color = Color(0xFFFFF2D7), style = MaterialTheme.typography.titleMedium)
            Text("Unknown app install: ${if (hasInstall) "OK" else "Missing"}", color = Color(0xFFFFF2D7))
            Text("All files access: ${if (hasFiles) "OK" else "Missing"}", color = Color(0xFFFFF2D7))
            Text(
                "Free space: ${humanBytes(freeBytes)} / Required: ${humanBytes(minBytes)}",
                color = Color(0xFFFFF2D7),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = onOpenInstallSettings) { Text("Install Settings") }
                OutlinedButton(onClick = onOpenFilesSettings) { Text("Storage Settings") }
                Button(onClick = onRefresh) { Text("Recheck") }
            }
        }
    }
}

@Composable
private fun StatusPill(label: String, color: Color) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(color.copy(alpha = 0.2f))
            .border(1.dp, color, RoundedCornerShape(999.dp))
            .padding(horizontal = 8.dp, vertical = 2.dp),
    ) {
        Text(
            text = label,
            color = color,
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
        )
    }
}

private fun humanBytes(value: Long): String {
    if (value <= 0) return "0 B"
    val units = listOf("B", "KB", "MB", "GB", "TB")
    var bytes = value.toDouble()
    var idx = 0
    while (bytes >= 1024 && idx < units.lastIndex) {
        bytes /= 1024
        idx += 1
    }
    return String.format(Locale.US, "%.1f %s", bytes, units[idx])
}
