package dev.veteran.quest.app.service

import java.io.File

interface ExtractionService {
    suspend fun ensureExtractorReady(): Result<File>
    suspend fun extract7z(archivePath: File, outputDir: File, password: String?): Result<Unit>
}
