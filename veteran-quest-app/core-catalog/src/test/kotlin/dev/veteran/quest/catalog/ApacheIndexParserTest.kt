package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import org.junit.Test

class ApacheIndexParserTest {
    @Test
    fun `parseEntries extracts href links from index html`() {
        val html = """
            <html><body><pre>
            <a href="../">../</a>
            <a href="meta.7z">meta.7z</a>
            <a href="040fa...001">040fa...001</a>
            </pre></body></html>
        """.trimIndent()

        val entries = ApacheIndexParser.parseEntries(html)

        assertThat(entries).containsExactly("meta.7z", "040fa...001")
    }
}
