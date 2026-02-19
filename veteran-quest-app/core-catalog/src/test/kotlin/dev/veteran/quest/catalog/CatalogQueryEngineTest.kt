package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.model.SortBy
import org.junit.Test

class CatalogQueryEngineTest {
    @Test
    fun `query supports package search prefix`() {
        val games = listOf(
            Game("A", "A", "pkg.a", "1"),
            Game("B", "B", "pkg.b", "1"),
        )

        val result = CatalogQueryEngine.query(
            games = games,
            allVersions = games,
            query = LibraryQuery(search = "pkg:pkg.b")
        )

        assertThat(result).hasSize(1)
        assertThat(result.single().packageName).isEqualTo("pkg.b")
    }

    @Test
    fun `query filters non mods`() {
        val games = listOf(
            Game("Beat Saber", "Beat Saber v1 -MOD", "pkg.mod", "1"),
            Game("Walkabout", "Walkabout v2", "pkg.clean", "2"),
        )

        val result = CatalogQueryEngine.query(
            games = games,
            allVersions = games,
            query = LibraryQuery(filter = LibraryFilter.NON_MODS, sortBy = SortBy.NAME)
        )

        assertThat(result.map { it.packageName }).containsExactly("pkg.clean")
    }
}
