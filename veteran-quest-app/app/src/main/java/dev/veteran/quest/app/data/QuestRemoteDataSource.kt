package dev.veteran.quest.app.data

import dev.veteran.quest.catalog.ConfigParser
import dev.veteran.quest.model.PublicConfig
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.File
import java.io.IOException
import java.time.Duration

class QuestRemoteDataSource(
    private val client: OkHttpClient = OkHttpClient.Builder()
        .callTimeout(Duration.ofMinutes(30))
        .build(),
) {
    companion object {
        private const val USER_AGENT = "rclone/v1.73.0"
        private val CONFIG_URLS = listOf(
            "https://raw.githubusercontent.com/vrpyou/quest/main/vrp-public.json",
            "https://vrpirates.wiki/downloads/vrp-public.json",
        )
    }

    data class HeadResult(
        val contentLength: Long,
        val acceptRanges: Boolean,
    )

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

    fun head(url: String): HeadResult {
        val request = Request.Builder()
            .url(url)
            .head()
            .header("User-Agent", USER_AGENT)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("HEAD failed for $url with ${response.code}")
            }

            return HeadResult(
                contentLength = response.header("Content-Length")?.toLongOrNull() ?: -1L,
                acceptRanges = response.header("Accept-Ranges")?.contains("bytes", ignoreCase = true) == true,
            )
        }
    }

    fun headExists(url: String): Boolean = runCatching { head(url) }.isSuccess

    fun downloadToFile(
        url: String,
        destination: File,
        resume: Boolean,
        onChunk: (bytesRead: Long) -> Unit,
    ) {
        val existingLength = if (resume && destination.exists()) destination.length() else 0L

        val requestBuilder = Request.Builder()
            .url(url)
            .header("User-Agent", USER_AGENT)

        if (existingLength > 0L) {
            requestBuilder.header("Range", "bytes=$existingLength-")
        }

        val request = requestBuilder.build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Download failed for $url with ${response.code}")
            }

            val append = existingLength > 0L && response.code == 206
            destination.parentFile?.mkdirs()

            response.body?.byteStream()?.use { input ->
                destination.outputStream().use { existing ->
                    if (append) {
                        existing.channel.position(destination.length())
                    } else {
                        existing.channel.truncate(0)
                    }
                    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                    while (true) {
                        val read = input.read(buffer)
                        if (read <= 0) {
                            break
                        }
                        existing.write(buffer, 0, read)
                        onChunk(read.toLong())
                    }
                }
            } ?: throw IOException("Empty response body for $url")
        }
    }
}
