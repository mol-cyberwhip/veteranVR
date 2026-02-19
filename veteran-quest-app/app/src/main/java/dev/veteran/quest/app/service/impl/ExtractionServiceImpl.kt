package dev.veteran.quest.app.service.impl

import android.content.Context
import dev.veteran.quest.app.service.ExtractionService
import dev.veteran.quest.app.util.AppPaths
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.apache.commons.compress.utils.MultiReadOnlySeekableByteChannel
import java.io.File
import java.io.FileOutputStream
import java.nio.channels.SeekableByteChannel
import java.nio.file.Files
import java.nio.file.StandardOpenOption

class ExtractionServiceImpl(
    private val context: Context,
) : ExtractionService {
    override suspend fun ensureExtractorReady(): Result<File> = withContext(Dispatchers.IO) {
        runCatching { AppPaths.binariesRoot(context).apply { mkdirs() } }
    }

    override suspend fun extract7z(archivePath: File, outputDir: File, password: String?): Result<Unit> =
        withContext(Dispatchers.IO) {
            runCatching {
                check(archivePath.exists()) { "Archive not found: ${archivePath.absolutePath}" }
                outputDir.mkdirs()
                val archiveParts = resolveArchiveParts(archivePath)

                openSevenZipFile(archiveParts, password).use { sevenZ ->
                    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                    while (true) {
                        val entry = sevenZ.nextEntry ?: break
                        val destination = File(outputDir, entry.name)
                        if (entry.isDirectory) {
                            destination.mkdirs()
                            continue
                        }

                        destination.parentFile?.mkdirs()
                        FileOutputStream(destination).use { output ->
                            while (true) {
                                val read = sevenZ.read(buffer)
                                if (read <= 0) {
                                    break
                                }
                                output.write(buffer, 0, read)
                            }
                        }
                    }
                }
            }.recoverCatching { error ->
                val normalized = (error.message ?: "").lowercase()
                val message = when {
                    "password" in normalized || "encrypted" in normalized || "coder" in normalized -> {
                        "Wrong archive password"
                    }
                    "not enough space" in normalized || "no space left" in normalized -> {
                        "Insufficient storage for extraction"
                    }
                    "data error" in normalized || "unexpected end" in normalized || "corrupt" in normalized -> {
                        "Archive appears corrupt"
                    }
                    else -> "Extraction failed"
                }
                throw IllegalStateException(message, error)
            }
        }

    private fun resolveArchiveParts(archivePath: File): List<File> {
        val name = archivePath.name
        val dir = archivePath.parentFile ?: return listOf(archivePath)
        val multipartRegex = Regex(""".*\.7z\.\d{3}$""", RegexOption.IGNORE_CASE)
        if (!multipartRegex.matches(name)) {
            return listOf(archivePath)
        }

        val prefix = name.substringBeforeLast(".")
        val parts = dir.listFiles()
            ?.filter { file ->
                file.name.startsWith("$prefix.") &&
                    file.name.substringAfterLast(".").matches(Regex("""\d{3}"""))
            }
            ?.sortedBy { it.name }
            .orEmpty()

        return if (parts.isNotEmpty()) parts else listOf(archivePath)
    }

    private fun openSevenZipFile(parts: List<File>, password: String?): SevenZFile {
        val channel: SeekableByteChannel = if (parts.size == 1) {
            Files.newByteChannel(parts.first().toPath(), StandardOpenOption.READ)
        } else {
            MultiReadOnlySeekableByteChannel.forFiles(*parts.toTypedArray())
        }

        val builder = SevenZFile.builder()
            .setSeekableByteChannel(channel)

        if (!password.isNullOrBlank()) {
            builder.setPassword(password.toCharArray())
        }
        return builder.get()
    }
}
