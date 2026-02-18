package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import org.junit.Test

class RemoteCatalogPlannerTest {
    @Test
    fun `metaArchiveUrl normalizes trailing slash`() {
        assertThat(RemoteCatalogPlanner.metaArchiveUrl("https://example.com/")).isEqualTo("https://example.com/meta.7z")
    }

    @Test
    fun `gameChunkUrls includes only matching chunk files`() {
        val hash = "abc123"
        val html = """
            <a href="../">../</a>
            <a href="abc123.7z.001">abc123.7z.001</a>
            <a href="abc123.7z.002">abc123.7z.002</a>
            <a href="different.7z.001">different.7z.001</a>
            <a href="abc123.txt">abc123.txt</a>
        """.trimIndent()

        val result = RemoteCatalogPlanner.gameChunkUrls("https://files.example", hash, html)

        assertThat(result.map { it.name }).containsExactly("abc123.7z.001", "abc123.7z.002").inOrder()
        assertThat(result.first().url).isEqualTo("https://files.example/abc123/abc123.7z.001")
    }
}
