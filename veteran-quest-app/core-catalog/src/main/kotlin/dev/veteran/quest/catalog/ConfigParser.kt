package dev.veteran.quest.catalog

import dev.veteran.quest.model.PublicConfig
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.util.Base64

object ConfigParser {
    private val json = Json { ignoreUnknownKeys = true }

    fun parse(rawJson: String): PublicConfig {
        val root = json.parseToJsonElement(rawJson).jsonObject
        val baseUri = root["baseUri"]?.jsonPrimitive?.content ?: root["base_uri"]?.jsonPrimitive?.content.orEmpty()
        val encodedPassword = root["password"]?.jsonPrimitive?.content.orEmpty()
        val decodedPassword = String(Base64.getDecoder().decode(encodedPassword), Charsets.UTF_8)

        return PublicConfig(
            baseUri = baseUri,
            password = decodedPassword,
        )
    }
}
