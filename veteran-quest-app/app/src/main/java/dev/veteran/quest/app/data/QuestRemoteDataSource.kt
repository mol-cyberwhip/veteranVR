package dev.veteran.quest.app.data

import dev.veteran.quest.catalog.ConfigParser
import dev.veteran.quest.model.PublicConfig
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.IOException

class QuestRemoteDataSource(
    private val client: OkHttpClient = OkHttpClient(),
) {
    companion object {
        private const val USER_AGENT = "rclone/v1.73.0"
        private val CONFIG_URLS = listOf(
            "https://raw.githubusercontent.com/vrpyou/quest/main/vrp-public.json",
            "https://vrpirates.wiki/downloads/vrp-public.json",
        )
    }

    fun fetchPublicConfig(): PublicConfig {
        var lastError: Throwable? = null
        for (url in CONFIG_URLS) {
            try {
                val request = Request.Builder()
                    .url(url)
                    .header("User-Agent", USER_AGENT)
                    .build()

                client.newCall(request).execute().use { response ->
                    if (!response.isSuccessful) {
                        throw IOException("Config fetch failed for $url with ${response.code}")
                    }
                    val body = response.body?.string().orEmpty()
                    return ConfigParser.parse(body)
                }
            } catch (t: Throwable) {
                lastError = t
            }
        }

        throw IOException("Could not fetch public config", lastError)
    }

    fun fetchText(url: String): String {
        val request = Request.Builder()
            .url(url)
            .header("User-Agent", USER_AGENT)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Fetch failed for $url with ${response.code}")
            }
            return response.body?.string().orEmpty()
        }
    }

    fun headExists(url: String): Boolean {
        val request = Request.Builder()
            .url(url)
            .head()
            .header("User-Agent", USER_AGENT)
            .build()

        client.newCall(request).execute().use { response ->
            return response.isSuccessful
        }
    }
}
