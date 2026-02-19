package dev.veteran.quest.app.ui

import app.cash.paparazzi.DeviceConfig
import app.cash.paparazzi.Paparazzi
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.ui.components.CatalogGrid
import dev.veteran.quest.app.ui.components.DiagnosticsPanel
import dev.veteran.quest.app.ui.components.OperationStrip
import dev.veteran.quest.app.ui.components.QuestTopBar
import dev.veteran.quest.app.ui.components.SetupGateBanner
import dev.veteran.quest.app.ui.components.VeteranPanel
import dev.veteran.quest.app.ui.theme.VeteranQuestTheme
import dev.veteran.quest.model.Game
import androidx.compose.ui.unit.dp
import org.junit.Rule
import org.junit.Test

class QuestUiSnapshotTest {
    @get:Rule
    val paparazzi = Paparazzi(
        deviceConfig = DeviceConfig.PIXEL_6,
        theme = "android:Theme.DeviceDefault.NoActionBar",
    )

    @Test
    fun topBarIdle() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                QuestTopBar(
                    message = "Catalog ready (120 titles, cache)",
                    operationLabel = null,
                    queueCount = 0,
                )
            }
        }
    }

    @Test
    fun setupGateWarning() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                SetupGateBanner(
                    hasInstall = false,
                    hasFiles = false,
                    freeBytes = 5L * 1024L * 1024L * 1024L,
                    minBytes = 6L * 1024L * 1024L * 1024L,
                    onOpenInstallSettings = {},
                    onOpenFilesSettings = {},
                    onRefresh = {},
                    humanBytes = { "$it B" },
                )
            }
        }
    }

    @Test
    fun operationStripDownloading() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                OperationStrip(
                    operation = sampleOperation,
                    onPause = {},
                    onResume = {},
                )
            }
        }
    }

    @Test
    fun catalogGridMixedState() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                VeteranPanel {
                    CatalogGrid(
                        items = sampleItems,
                        operations = listOf(sampleOperation),
                        cardGap = 8.dp,
                        thumbHeight = 110.dp,
                        onInstall = {},
                        onUninstall = {},
                    )
                }
            }
        }
    }

    @Test
    fun diagnosticsCollapsed() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                DiagnosticsPanel(
                    logs = sampleLogs,
                    showDiagnostics = false,
                    onShowDiagnosticsChanged = {},
                )
            }
        }
    }

    @Test
    fun diagnosticsExpanded() {
        paparazzi.snapshot {
            VeteranQuestTheme {
                DiagnosticsPanel(
                    logs = sampleLogs,
                    showDiagnostics = true,
                    onShowDiagnosticsChanged = {},
                )
            }
        }
    }

    private val sampleOperation = DownloadOperation(
        operationId = "op-77",
        packageName = "com.veteran.sample",
        releaseName = "Sample Arena v12",
        state = DownloadState.DOWNLOADING,
        progressPercent = 63.0,
        bytesDone = 123,
        bytesTotal = 456,
        speedBps = 1200,
        etaSeconds = 15,
    )

    private val sampleItems = listOf(
        LibraryItemUi(
            game = Game(
                gameName = "Arena Clash",
                releaseName = "Arena Clash v1.2",
                packageName = "com.veteran.arena",
                versionCode = "12",
                size = "2.1 GB",
                popularityRank = 1,
                isNew = true,
            ),
            thumbnailPath = "",
            thumbnailExists = false,
            notePath = "",
            noteExists = false,
            isInstalled = true,
        ),
        LibraryItemUi(
            game = Game(
                gameName = "Synth Mod",
                releaseName = "Synth Modded Pack",
                packageName = "com.veteran.synth",
                versionCode = "4",
                size = "1.4 GB",
            ),
            thumbnailPath = "",
            thumbnailExists = false,
            notePath = "",
            noteExists = false,
            isInstalled = false,
        ),
    )

    private val sampleLogs = listOf(
        OperationLogEntry(
            timestampMs = 1,
            operationId = "op-77",
            stage = "download",
            level = "INFO",
            message = "Connecting to host",
        ),
        OperationLogEntry(
            timestampMs = 2,
            operationId = "op-77",
            stage = "install",
            level = "WARN",
            message = "Range unsupported, restarting chunk",
        ),
        OperationLogEntry(
            timestampMs = 3,
            operationId = "op-77",
            stage = "extract",
            level = "ERROR",
            message = "Archive password invalid",
        ),
    )
}
