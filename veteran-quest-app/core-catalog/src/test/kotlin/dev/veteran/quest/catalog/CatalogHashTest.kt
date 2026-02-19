package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import org.junit.Test

class CatalogHashTest {
    @Test
    fun `hash matches expected md5 format`() {
        val hash = CatalogHash.gameNameToHash("17 Seconds v2+1.0 -VRP")
        assertThat(hash).hasLength(32)
        assertThat(hash).matches("[a-f0-9]{32}")
    }
}
