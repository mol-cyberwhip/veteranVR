package dev.veteran.quest.app.service

import dev.veteran.quest.app.model.InstallTxtExecutionReport
import dev.veteran.quest.installer.UninstallOptions
import java.io.File

interface PackageInstallService {
    suspend fun installFromExtractedGameDir(
        operationId: String,
        gameDir: File,
        packageName: String,
    ): Result<InstallTxtExecutionReport>

    suspend fun uninstall(
        operationId: String,
        packageName: String,
        options: UninstallOptions,
    ): Result<Unit>
}
