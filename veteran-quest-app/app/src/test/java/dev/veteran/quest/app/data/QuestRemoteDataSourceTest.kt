package dev.veteran.quest.app.data

import com.google.common.truth.Truth.assertThat
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import okio.Buffer
import org.junit.After
import org.junit.Before
import org.junit.Test
import java.io.File
import java.nio.file.Files

class QuestRemoteDataSourceTest {
    private lateinit var server: MockWebServer
    private lateinit var remote: QuestRemoteDataSource

    @Before
    fun setUp() {
        server = MockWebServer()
        remote = QuestRemoteDataSource()
    }

    @After
    fun tearDown() {
        server.shutdown()
    }

    @Test
    fun `downloadToFile handles 416 when local file is already complete`() {
        val payload = "DATA".toByteArray()
        server.dispatcher = object : Dispatcher() {
            override fun dispatch(request: RecordedRequest): MockResponse {
                if (request.method == "HEAD") {
                    return MockResponse()
                        .setResponseCode(200)
                        .setHeader("Content-Length", payload.size.toString())
                        .setHeader("Accept-Ranges", "bytes")
                }
                if (request.getHeader("Range") != null) {
                    return MockResponse()
                        .setResponseCode(416)
                        .setHeader("Content-Range", "bytes */${payload.size}")
                }
                return MockResponse()
                    .setResponseCode(200)
                    .setBody(Buffer().write(payload))
            }
        }
        server.start()

        val output = tempFile("already-complete.apk")
        output.writeBytes(payload)
        var streamedBytes = 0L

        remote.downloadToFile(
            url = server.url("/meta.7z").toString(),
            destination = output,
            resume = true,
            onChunk = { streamedBytes += it },
        )

        assertThat(output.readBytes()).isEqualTo(payload)
        assertThat(streamedBytes).isEqualTo(0L)
    }

    @Test
    fun `downloadToFile retries full download after 416 when local file is stale`() {
        val payload = "abc".toByteArray()
        server.dispatcher = object : Dispatcher() {
            override fun dispatch(request: RecordedRequest): MockResponse {
                if (request.method == "HEAD") {
                    return MockResponse()
                        .setResponseCode(200)
                        .setHeader("Content-Length", payload.size.toString())
                        .setHeader("Accept-Ranges", "bytes")
                }
                if (request.getHeader("Range") != null) {
                    return MockResponse()
                        .setResponseCode(416)
                        .setHeader("Content-Range", "bytes */${payload.size}")
                }
                return MockResponse()
                    .setResponseCode(200)
                    .setBody(Buffer().write(payload))
            }
        }
        server.start()

        val output = tempFile("stale-local.apk")
        output.writeText("stale-local-content")
        var streamedBytes = 0L

        remote.downloadToFile(
            url = server.url("/meta.7z").toString(),
            destination = output,
            resume = true,
            onChunk = { streamedBytes += it },
        )

        assertThat(output.readBytes()).isEqualTo(payload)
        assertThat(streamedBytes).isEqualTo(payload.size.toLong())
    }

    private fun tempFile(name: String): File {
        val dir = Files.createTempDirectory("quest-remote-test").toFile()
        dir.deleteOnExit()
        return File(dir, name).also { it.deleteOnExit() }
    }
}
