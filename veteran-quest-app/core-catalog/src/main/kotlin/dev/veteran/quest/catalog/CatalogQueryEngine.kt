package dev.veteran.quest.catalog

import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.model.SortBy

object CatalogQueryEngine {
    fun query(games: List<Game>, allVersions: List<Game>, query: LibraryQuery): List<Game> {
        var filtered = search(games, allVersions, query.search)

        filtered = when (query.filter) {
            LibraryFilter.ALL -> filtered
            LibraryFilter.FAVORITES -> filtered.filter { it.packageName in query.favorites }
            LibraryFilter.NEW -> filtered.filter { it.isNew }
            LibraryFilter.POPULAR -> filtered.filter { it.popularityRank in 1..200 }
            LibraryFilter.NON_MODS -> filtered.filterNot { it.isModded }
        }

        filtered = sort(filtered, query.sortBy)
        return if (query.sortAscending) filtered else filtered.reversed()
    }

    private fun search(games: List<Game>, allVersions: List<Game>, query: String): List<Game> {
        val q = query.trim()
        if (q.isEmpty()) {
            return games
        }

        val releasePrefix = "release:"
        if (q.startsWith(releasePrefix, ignoreCase = true)) {
            val needle = q.removePrefix(releasePrefix).trim().lowercase()
            return allVersions.filter { it.releaseName.lowercase().contains(needle) }
        }

        val packagePrefix = "pkg:"
        if (q.startsWith(packagePrefix, ignoreCase = true)) {
            val needle = q.removePrefix(packagePrefix).trim().lowercase()
            return allVersions.filter { it.packageName.lowercase().contains(needle) }
        }

        val needle = q.lowercase()
        return games.filter {
            it.gameName.lowercase().contains(needle) ||
                it.releaseName.lowercase().contains(needle) ||
                it.packageName.lowercase().contains(needle)
        }
    }

    private fun sort(games: List<Game>, sortBy: SortBy): List<Game> {
        return when (sortBy) {
            SortBy.NAME -> games.sortedBy { it.gameName.lowercase() }
            SortBy.DATE -> games.sortedBy { it.lastUpdated }
            SortBy.SIZE -> games.sortedBy { parseSizeMb(it.size) }
            SortBy.POPULARITY -> games.sortedBy {
                if (it.popularityRank > 0) {
                    it.popularityRank
                } else {
                    Int.MAX_VALUE
                }
            }
        }
    }

    private fun parseSizeMb(size: String): Double {
        val normalized = size.trim().lowercase()
        if (normalized.isEmpty() || normalized == "unknown") {
            return 0.0
        }

        fun parseWithUnit(unit: String, factor: Double): Double? {
            val idx = normalized.indexOf(unit)
            if (idx < 0) {
                return null
            }
            val number = normalized.substring(0, idx).trim().toDoubleOrNull() ?: return null
            return number * factor
        }

        return parseWithUnit("gb", 1024.0)
            ?: parseWithUnit("mb", 1.0)
            ?: parseWithUnit("kb", 1.0 / 1024.0)
            ?: normalized.toDoubleOrNull()
            ?: 0.0
    }
}
