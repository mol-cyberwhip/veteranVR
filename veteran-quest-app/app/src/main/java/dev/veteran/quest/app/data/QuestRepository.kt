package dev.veteran.quest.app.data

import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.installer.UninstallOptions

interface QuestRepository {
    suspend fun syncCatalog(force: Boolean): Result<Int>
    suspend fun getLibrary(query: LibraryQuery): List<Game>
    suspend fun install(game: Game): Result<Unit>
    suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit>
}
