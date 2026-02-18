package dev.veteran.quest.app.data

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import dev.veteran.quest.app.model.CatalogSnapshot
import dev.veteran.quest.app.model.CatalogSyncSummary
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogLevel
import dev.veteran.quest.app.service.CatalogSyncService
import dev.veteran.quest.app.service.DownloadQueueService
import dev.veteran.quest.app.service.OperationLogService
import dev.veteran.quest.app.service.PackageInstallService
import dev.veteran.quest.app.service.PermissionGateService
import dev.veteran.quest.app.util.AppPaths
import dev.veteran.quest.catalog.CatalogQueryEngine
import dev.veteran.quest.installer.UninstallOptions
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.UUID

class QuestRepositoryImpl(
    private val context: Context,
    private val catalogSyncService: CatalogSyncService,
    private val downloadQueueService: DownloadQueueService,
    private val packageInstallService: PackageInstallService,
    private val logService: OperationLogService,
    private val permissionGateService: PermissionGateService,
) : QuestRepository {
    private val lock = Mutex()
    private var snapshot: CatalogSnapshot? = null

    override val operations = downloadQueueService.operations
    override val logs: StateFlow<List<dev.veteran.quest.app.model.OperationLogEntry>> = logService.logs

    override suspend fun syncCatalog(force: Boolean): Result<CatalogSyncSummary> {
        val result = catalogSyncService.syncCatalog(force)
        result.onSuccess { latest ->
            lock.withLock {
                snapshot = latest
            }
        }.onFailure { err ->
            logService.append("catalog-sync", "sync", OperationLogLevel.ERROR, "Catalog sync failed", err.message)
        }
        return result.map { it.summary }
    }

    override suspend fun getLibrary(query: LibraryQuery): List<LibraryItemUi> {
        val current = lock.withLock {
            snapshot
        } ?: return emptyList()

        val games = CatalogQueryEngine.query(current.dataset.latestGames, current.dataset.allVersions, query)
        val thumbsDir = AppPaths.cacheThumbnailsDir(context)
        val notesDir = AppPaths.cacheNotesDir(context)

        return games.map { game ->
            val thumb = java.io.File(thumbsDir, "${game.packageName}.jpg")
            val note = java.io.File(notesDir, "${game.packageName}.txt")

            LibraryItemUi(
                game = game,
                thumbnailPath = thumb.absolutePath,
                thumbnailExists = thumb.exists(),
                notePath = note.absolutePath,
                noteExists = note.exists(),
                isInstalled = isInstalled(game.packageName),
            )
        }
    }

    override suspend fun enqueueInstall(game: Game): Result<String> {
        val current = lock.withLock { snapshot }
        if (current == null) {
            return Result.failure(IllegalStateException("Catalog not synced yet"))
        }
        return downloadQueueService.enqueueInstall(game)
    }

    override suspend fun pauseDownload(operationId: String): Result<Unit> {
        return downloadQueueService.pause(operationId)
    }

    override suspend fun resumeDownload(operationId: String): Result<Unit> {
        return downloadQueueService.resume(operationId)
    }

    override suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit> {
        val opId = "uninstall-${UUID.randomUUID()}"
        return packageInstallService.uninstall(opId, packageName, options)
    }

    override suspend fun permissionStatus() = permissionGateService.evaluate()

    suspend fun currentConfigProvider() = lock.withLock {
        snapshot?.config ?: throw IllegalStateException("Catalog config unavailable; sync first")
    }

    private fun isInstalled(packageName: String): Boolean {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                context.packageManager.getPackageInfo(packageName, PackageManager.PackageInfoFlags.of(0L))
            } else {
                @Suppress("DEPRECATION")
                context.packageManager.getPackageInfo(packageName, 0)
            }
            true
        } catch (_: Throwable) {
            false
        }
    }
}
