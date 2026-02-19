package dev.veteran.quest.app.data

import dev.veteran.quest.catalog.CatalogParser
import dev.veteran.quest.catalog.CatalogQueryEngine
import dev.veteran.quest.catalog.RemoteCatalogPlanner
import dev.veteran.quest.app.model.CatalogSyncSummary
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.PermissionGateStatus
import dev.veteran.quest.installer.UninstallOptions
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext

private const val BOOTSTRAP_CATALOG = """
Game Name;Release Name;Package Name;Version Code;Last Updated;Size (MB);Downloads;Rating;Rating Count
Beat Saber;Beat Saber v193+1.39.0 -MOD;com.beatgames.beatsaber;193;2026-01-12 10:00 UTC;1550;99.2;0;0
Walkabout Mini Golf;Walkabout Mini Golf v182+5.3;com.mightycoconut.WMG;182;2026-01-18 07:10 UTC;990;98.6;0;0
Dungeons of Eternity;Dungeons of Eternity v75+1.2.1;com.othergate.dungeonsofeternity;75;2025-12-03 11:45 UTC;4200;97.1;0;0
Puzzling Places;Puzzling Places v150+2.6.0;com.realitiesio.puzzlingplaces;150;2025-10-03 04:01 UTC;1700;95.4;0;0
"""

class BootstrapQuestRepository(
    private val remote: QuestRemoteDataSource = QuestRemoteDataSource(),
) : QuestRepository {
    override val operations: StateFlow<List<DownloadOperation>> = MutableStateFlow(emptyList())
    override val logs: StateFlow<List<OperationLogEntry>> = MutableStateFlow(emptyList())
    private val lock = Mutex()
    private var dataset = CatalogParser.parseGameList(BOOTSTRAP_CATALOG.trimIndent())

    override suspend fun syncCatalog(force: Boolean): Result<CatalogSyncSummary> {
        delay(150)
        return withContext(Dispatchers.IO) {
            runCatching {
                val config = remote.fetchPublicConfig()
                val metaUrl = RemoteCatalogPlanner.metaArchiveUrl(config.baseUri)
                // Temporary real-network check: this confirms remote config and base URI are reachable
                // with expected headers while download/extract implementation is being built.
                remote.fetchText(config.baseUri)
                check(remote.headExists(metaUrl)) { "meta.7z not reachable at $metaUrl" }

                lock.withLock {
                    if (force) {
                        // TODO: force will trigger full cache bust once local cache is implemented.
                    }
                    dataset = CatalogParser.parseGameList(BOOTSTRAP_CATALOG.trimIndent())
                    CatalogSyncSummary(
                        lastSyncEpochMs = System.currentTimeMillis(),
                        gamesCount = dataset.latestGames.size,
                        usedCache = false,
                    )
                }
            }
        }
    }

    override suspend fun getLibrary(query: LibraryQuery): List<LibraryItemUi> {
        return lock.withLock {
            CatalogQueryEngine.query(dataset.latestGames, dataset.allVersions, query).map { game ->
                LibraryItemUi(
                    game = game,
                    thumbnailPath = "",
                    thumbnailExists = false,
                    notePath = "",
                    noteExists = false,
                    isInstalled = false,
                )
            }
        }
    }

    override suspend fun enqueueInstall(game: Game): Result<String> {
        delay(120)
        return Result.success("bootstrap-op")
    }

    override suspend fun pauseDownload(operationId: String): Result<Unit> {
        return Result.success(Unit)
    }

    override suspend fun resumeDownload(operationId: String): Result<Unit> {
        return Result.success(Unit)
    }

    override suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit> {
        delay(120)
        if (options.keepObb || options.keepData) {
            // Kept for API parity with the real uninstall implementation.
        }
        return Result.success(Unit)
    }

    override suspend fun permissionStatus(): PermissionGateStatus {
        return PermissionGateStatus(
            canInstallPackages = false,
            hasAllFilesAccess = false,
            freeBytes = 0,
            minRequiredBytes = 1,
        )
    }
}
