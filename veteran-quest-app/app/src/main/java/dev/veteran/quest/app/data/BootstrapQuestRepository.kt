package dev.veteran.quest.app.data

import dev.veteran.quest.catalog.CatalogParser
import dev.veteran.quest.catalog.CatalogQueryEngine
import dev.veteran.quest.installer.UninstallOptions
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import kotlinx.coroutines.delay
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

private const val BOOTSTRAP_CATALOG = """
Game Name;Release Name;Package Name;Version Code;Last Updated;Size (MB);Downloads;Rating;Rating Count
Beat Saber;Beat Saber v193+1.39.0 -MOD;com.beatgames.beatsaber;193;2026-01-12 10:00 UTC;1550;99.2;0;0
Walkabout Mini Golf;Walkabout Mini Golf v182+5.3;com.mightycoconut.WMG;182;2026-01-18 07:10 UTC;990;98.6;0;0
Dungeons of Eternity;Dungeons of Eternity v75+1.2.1;com.othergate.dungeonsofeternity;75;2025-12-03 11:45 UTC;4200;97.1;0;0
Puzzling Places;Puzzling Places v150+2.6.0;com.realitiesio.puzzlingplaces;150;2025-10-03 04:01 UTC;1700;95.4;0;0
"""

class BootstrapQuestRepository : QuestRepository {
    private val lock = Mutex()
    private var dataset = CatalogParser.parseGameList(BOOTSTRAP_CATALOG.trimIndent())

    override suspend fun syncCatalog(force: Boolean): Result<Int> {
        delay(350)
        return lock.withLock {
            // Initial scaffold: this uses a baked-in catalog so UI flow is ready while
            // download/extract/install internals are implemented in subsequent steps.
            if (force) {
                // Force path is a no-op for the bootstrap repository.
            }
            dataset = CatalogParser.parseGameList(BOOTSTRAP_CATALOG.trimIndent())
            Result.success(dataset.latestGames.size)
        }
    }

    override suspend fun getLibrary(query: LibraryQuery): List<Game> {
        return lock.withLock {
            CatalogQueryEngine.query(dataset.latestGames, dataset.allVersions, query)
        }
    }

    override suspend fun install(game: Game): Result<Unit> {
        delay(120)
        return Result.success(Unit)
    }

    override suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit> {
        delay(120)
        if (options.keepObb || options.keepData) {
            // Kept for API parity with the real uninstall implementation.
        }
        return Result.success(Unit)
    }
}
