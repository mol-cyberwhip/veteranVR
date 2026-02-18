package dev.veteran.quest.app.data

import dev.veteran.quest.app.model.CatalogSyncSummary
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.PermissionGateStatus
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.installer.UninstallOptions
import kotlinx.coroutines.flow.StateFlow

interface QuestRepository {
    val operations: StateFlow<List<DownloadOperation>>
    val logs: StateFlow<List<OperationLogEntry>>

    suspend fun syncCatalog(force: Boolean): Result<CatalogSyncSummary>
    suspend fun getLibrary(query: LibraryQuery): List<LibraryItemUi>
    suspend fun enqueueInstall(game: Game): Result<String>
    suspend fun pauseDownload(operationId: String): Result<Unit>
    suspend fun resumeDownload(operationId: String): Result<Unit>
    suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit>
    suspend fun permissionStatus(): PermissionGateStatus
}
