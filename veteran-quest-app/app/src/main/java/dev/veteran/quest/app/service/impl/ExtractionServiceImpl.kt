package dev.veteran.quest.app.service.impl

import android.content.Context
import dev.veteran.quest.app.service.ExtractionService
import dev.veteran.quest.app.util.AppPaths
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

class ExtractionServiceImpl(
    private val context: Context,
) : ExtractionService {
    private val binaryAssetName = "7zz"

    override suspend fun ensureExtractorReady(): Result<File> = withContext(Dispatchers.IO) {
        runCatching {
            val binDir = AppPaths.binariesRoot(context).apply { mkdirs() }
            val binary = File(binDir, binaryAssetName)

            if (!binary.exists()) {
                context.assets.open(binaryAssetName).use { input ->
                    binary.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
            }

            check(binary.setExecutable(true, true)) { "Failed to set executable bit on 7zz" }
            binary
        }
    }

    override suspend fun extract7z(archivePath: File, outputDir: File, password: String?): Result<Unit> =
        withContext(Dispatchers.IO) {
            runCatching {
                val extractor = ensureExtractorReady().getOrThrow()
                check(archivePath.exists()) { "Archive not found: ${archivePath.absolutePath}" }
                outputDir.mkdirs()

                val cmd = mutableListOf(
                    extractor.absolutePath,
                    "x",
                    "-o${outputDir.absolutePath}",
                    archivePath.absolutePath,
                    "-y",
                )

                if (!password.isNullOrBlank()) {
                    cmd += "-p$password"
                } else {
                    cmd += "-p"
                }

                val process = ProcessBuilder(cmd)
                    .redirectErrorStream(true)
                    .start()

                val output = process.inputStream.bufferedReader().use { it.readText() }
                val exit = process.waitFor()

                if (exit != 0) {
                    val normalized = output.lowercase()
                    val message = when {
                        "wrong password" in normalized || "can not open encrypted archive" in normalized -> {
                            "Wrong archive password"
                        }
                        "not enough space" in normalized || "no space left" in normalized -> {
                            "Insufficient storage for extraction"
                        }
                        "data error" in normalized || "unexpected end of archive" in normalized -> {
                            "Archive appears corrupt"
                        }
                        else -> "Extraction failed"
                    }
                    throw IllegalStateException("$message\n$output")
                }
            }
        }
}
