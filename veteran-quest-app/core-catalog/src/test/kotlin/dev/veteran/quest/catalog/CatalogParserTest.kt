package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.time.Instant

class CatalogParserTest {
    @Test
    fun `parse keeps highest version per package plus game name`() {
        val content = """
            Header
            Game A;RelA v1+1;pkg.a;10;2025-01-01 00:00 UTC;100;120
            Game A;RelA v2+2;pkg.a;11;2025-01-05 00:00 UTC;101;130
            Game B;RelB v1+1;pkg.b;2;2025-01-10 00:00 UTC;90;40
        """.trimIndent()

        val dataset = CatalogParser.parseGameList(content, now = Instant.parse("2025-01-15T00:00:00Z"))

        assertThat(dataset.latestGames).hasSize(2)
        val gameA = dataset.latestGames.first { it.packageName == "pkg.a" }
        assertThat(gameA.versionCode).isEqualTo("11")
        assertThat(gameA.popularityRank).isEqualTo(1)
    }

    @Test
    fun `parse marks recent games as new`() {
        val content = """
            Header
            Game A;RelA v1+1;pkg.a;10;2025-01-10 00:00 UTC;100;120
        """.trimIndent()

        val dataset = CatalogParser.parseGameList(content, now = Instant.parse("2025-01-20T00:00:00Z"))

        assertThat(dataset.latestGames.single().isNew).isTrue()
    }
}
