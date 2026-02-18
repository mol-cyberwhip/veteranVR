package dev.veteran.quest.catalog

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.util.Base64

class ConfigParserTest {
    @Test
    fun `parse decodes base64 password`() {
        val encoded = Base64.getEncoder().encodeToString("secret".toByteArray())

        val parsed = ConfigParser.parse(
            """
            {
              "baseUri": "https://example.com/",
              "password": "$encoded"
            }
            """.trimIndent()
        )

        assertThat(parsed.baseUri).isEqualTo("https://example.com/")
        assertThat(parsed.password).isEqualTo("secret")
    }
}
