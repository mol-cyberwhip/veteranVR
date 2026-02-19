package dev.veteran.quest.app.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.service.PermissionGateService
import dev.veteran.quest.app.ui.components.CatalogGrid
import dev.veteran.quest.app.ui.components.ControlRail
import dev.veteran.quest.app.ui.components.OperationStrip
import dev.veteran.quest.app.ui.components.QuestTopBar
import dev.veteran.quest.app.ui.components.SetupGateBanner
import dev.veteran.quest.app.ui.components.TelemetryBar
import dev.veteran.quest.app.ui.components.VeteranPanel
import dev.veteran.quest.app.ui.tokens.MotionProfile
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors
import dev.veteran.quest.app.ui.tokens.spacingForDensity
import java.util.Locale

@Composable
fun QuestAppScreen(
    viewModel: QuestViewModel,
    permissionGateService: PermissionGateService,
) {
    val state by viewModel.state.collectAsState()
    val context = LocalContext.current
    val view = LocalView.current
    val spacing = spacingForDensity(state.uiDensity)
    val animate = state.motionProfile == MotionProfile.SUBTLE

    DisposableEffect(state.activeOperation, state.keepAwakeDuringOps) {
        view.keepScreenOn = state.keepAwakeDuringOps && state.activeOperation != null
        onDispose {
            view.keepScreenOn = false
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(VeteranQuestColors.appBackgroundBrush),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(spacing.outerPadding),
            verticalArrangement = Arrangement.spacedBy(spacing.sectionGap),
        ) {
            QuestTopBar(
                message = state.message,
                operationLabel = state.activeOperationLabel,
                queueCount = state.operations.count { it.state.name != "COMPLETED" },
            )

            OperationStrip(
                operation = state.activeOperation,
                onPause = { viewModel.pauseOperation(it.operationId) },
                onResume = { viewModel.resumeOperation(it.operationId) },
            )

            AnimatedVisibility(
                visible = state.permissionStatus?.isReady == false,
                enter = if (animate) fadeIn() else fadeIn(),
                exit = if (animate) fadeOut() else fadeOut(),
            ) {
                val status = state.permissionStatus
                if (status != null) {
                    SetupGateBanner(
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
                        humanBytes = ::humanBytes,
                    )
                }
            }

            Row(
                modifier = Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(spacing.sectionGap),
            ) {
                ControlRail(
                    modifier = Modifier.width(spacing.railWidth),
                    search = state.search,
                    sortBy = state.sortBy,
                    sortAscending = state.sortAscending,
                    filter = state.filter,
                    syncing = state.syncing,
                    keepObb = state.keepObbOnUninstall,
                    keepData = state.keepDataOnUninstall,
                    keepAwake = state.keepAwakeDuringOps,
                    logs = state.logs,
                    showDiagnostics = state.showDiagnostics,
                    uiDensity = state.uiDensity,
                    motionProfile = state.motionProfile,
                    onSearch = viewModel::onSearchChanged,
                    onSort = viewModel::onSortByChanged,
                    onToggleSort = viewModel::onSortDirectionToggled,
                    onFilter = viewModel::onFilterChanged,
                    onSync = { viewModel.refreshCatalog(force = true) },
                    onKeepObb = viewModel::onKeepObbChanged,
                    onKeepData = viewModel::onKeepDataChanged,
                    onKeepAwake = viewModel::onKeepAwakeChanged,
                    onShowDiagnosticsChanged = viewModel::onShowDiagnosticsChanged,
                    onUiDensityChanged = viewModel::onUiDensityChanged,
                    onMotionProfileChanged = viewModel::onMotionProfileChanged,
                )

                VeteranPanel(modifier = Modifier.weight(1f), padding = spacing.panelPadding) {
                    CatalogGrid(
                        items = state.games,
                        operations = state.operations,
                        cardGap = spacing.cardGap,
                        thumbHeight = spacing.cardThumbHeight,
                        onInstall = { viewModel.enqueueInstall(it.game) },
                        onUninstall = { viewModel.uninstall(it.game) },
                    )
                }
            }

            TelemetryBar(
                activeOperation = state.activeOperation,
                queueCount = state.operations.count { it.state.name != "COMPLETED" },
                totalLogs = state.logs.size,
            )

            if (state.loading) {
                Spacer(modifier = Modifier.height(2.dp))
            }
        }
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
