package dev.veteran.quest.app.service.impl

import android.content.Context
import android.content.pm.PackageInstaller
import android.os.Environment
import dev.veteran.quest.app.model.InstallTxtExecutionReport
import dev.veteran.quest.app.model.OperationLogLevel
import dev.veteran.quest.app.platform.PackageInstallerBridge
import dev.veteran.quest.app.service.OperationLogService
import dev.veteran.quest.app.service.PackageInstallService
import dev.veteran.quest.installer.InstallAction
import dev.veteran.quest.installer.InstallScriptParser
import dev.veteran.quest.installer.UninstallOptions
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import java.io.File

class PackageInstallServiceImpl(
    private val context: Context,
    private val bridge: PackageInstallerBridge,
    private val logService: OperationLogService,
) : PackageInstallService {
    private val packageInstaller: PackageInstaller = context.packageManager.packageInstaller

    override suspend fun installFromExtractedGameDir(
        operationId: String,
        gameDir: File,
        packageName: String,
    ): Result<InstallTxtExecutionReport> = withContext(Dispatchers.IO) {
        runCatching {
            val installTxt = listOf("install.txt", "Install.txt")
                .map { File(gameDir, it) }
                .firstOrNull { it.exists() }

            if (installTxt != null) {
                logService.append(operationId, "install", OperationLogLevel.INFO, "Running install.txt")
                return@runCatching executeInstallTxt(operationId, installTxt, gameDir, packageName)
            }

            val apk = gameDir.listFiles()
                ?.filter { it.isFile && it.extension.equals("apk", ignoreCase = true) }
                ?.sortedBy { it.name }
                ?.firstOrNull()
                ?: throw IllegalStateException("No APK found in ${gameDir.absolutePath}")

            installApk(operationId, apk)

            val obbDir = File(gameDir, packageName)
            if (obbDir.isDirectory) {
                copyObb(operationId, packageName, obbDir)
            }

            InstallTxtExecutionReport(warnings = emptyList(), executedActions = 1)
        }
    }

    override suspend fun uninstall(
        operationId: String,
        packageName: String,
        options: UninstallOptions,
    ): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            val waiter = bridge.newWaiter(operationId)
            packageInstaller.uninstall(packageName, bridge.uninstallIntentSender(operationId))

            val result = withTimeout(10 * 60 * 1000L) { waiter.await() }
            result.getOrThrow()
            logService.append(operationId, "uninstall", OperationLogLevel.INFO, "Package uninstalled: $packageName")

            if (!options.keepObb) {
                val obbDir = File(Environment.getExternalStorageDirectory(), "Android/obb/$packageName")
                obbDir.deleteRecursively()
                logService.append(operationId, "cleanup", OperationLogLevel.INFO, "Removed OBB for $packageName")
            }

            if (!options.keepData) {
                val dataDir = File(Environment.getExternalStorageDirectory(), "Android/data/$packageName")
                dataDir.deleteRecursively()
                logService.append(operationId, "cleanup", OperationLogLevel.INFO, "Removed data for $packageName")
            }
        }
    }

    private suspend fun executeInstallTxt(
        operationId: String,
        installTxt: File,
        gameDir: File,
        packageName: String,
    ): InstallTxtExecutionReport {
        val parsed = InstallScriptParser.parse(installTxt.readText())
        val warnings = parsed.warnings.toMutableList()
        var executed = 0

        for (action in parsed.actions) {
            when (action) {
                is InstallAction.InstallApk -> {
                    val apk = File(gameDir, action.relativeApkPath)
                    if (!apk.exists()) {
                        warnings += "install.txt apk not found: ${action.relativeApkPath}"
                        continue
                    }
                    installApk(operationId, apk)
                    executed += 1
                }

                is InstallAction.PushDirectory -> {
                    val local = File(gameDir, action.relativeLocalPath)
                    if (!local.exists()) {
                        warnings += "install.txt source not found: ${action.relativeLocalPath}"
                        continue
                    }

                    if (!isAllowedPushDestination(action.remotePath)) {
                        warnings += "install.txt blocked push destination: ${action.remotePath}"
                        continue
                    }

                    val target = remotePathToFile(action.remotePath, packageName)
                    if (local.isDirectory) {
                        copyRecursively(local, target)
                    } else {
                        target.parentFile?.mkdirs()
                        local.copyTo(target, overwrite = true)
                    }
                    executed += 1
                }

                is InstallAction.Shell -> {
                    val shellResult = executeAllowedShell(operationId, packageName, action.command)
                    warnings += shellResult
                    if (shellResult.isEmpty()) {
                        executed += 1
                    }
                }
            }
        }

        logService.append(
            operationId,
            "install",
            if (warnings.isEmpty()) OperationLogLevel.INFO else OperationLogLevel.WARN,
            "install.txt completed",
            "executed=$executed warnings=${warnings.size}",
        )

        return InstallTxtExecutionReport(
            warnings = warnings,
            executedActions = executed,
        )
    }

    private suspend fun installApk(operationId: String, apk: File) {
        val params = PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL)
        val sessionId = packageInstaller.createSession(params)
        val session = packageInstaller.openSession(sessionId)

        session.openWrite("base.apk", 0, apk.length()).use { out ->
            apk.inputStream().use { input ->
                input.copyTo(out)
            }
            session.fsync(out)
        }

        val waiter = bridge.newWaiter(operationId)
        session.commit(bridge.installIntentSender(operationId))
        session.close()

        val result = withTimeout(10 * 60 * 1000L) { waiter.await() }
        result.getOrThrow()
        logService.append(operationId, "install", OperationLogLevel.INFO, "Installed APK ${apk.name}")
    }

    private suspend fun copyObb(operationId: String, packageName: String, localObbDir: File) {
        val destination = File(Environment.getExternalStorageDirectory(), "Android/obb/$packageName")
        if (destination.exists()) {
            destination.deleteRecursively()
        }
        copyRecursively(localObbDir, destination)
        logService.append(operationId, "install", OperationLogLevel.INFO, "Copied OBB for $packageName")
    }

    private fun isAllowedPushDestination(remotePath: String): Boolean {
        val normalized = remotePath.trim()
        return normalized.startsWith("/sdcard/Android/obb/") || normalized.startsWith("/sdcard/Android/data/")
    }

    private fun remotePathToFile(remotePath: String, packageName: String): File {
        val cleaned = remotePath.removePrefix("/sdcard/").trimStart('/')
        val root = Environment.getExternalStorageDirectory()
        val destination = File(root, cleaned)
        if (!destination.path.contains("/Android/obb/") && !destination.path.contains("/Android/data/")) {
            throw IllegalStateException("Blocked remote path: $remotePath")
        }
        if (destination.name == packageName || destination.path.endsWith("/$packageName")) {
            return destination
        }
        return destination
    }

    private suspend fun executeAllowedShell(operationId: String, packageName: String, command: String): List<String> {
        val tokens = command.split(Regex("\\s+")).filter { it.isNotBlank() }
        if (tokens.isEmpty()) {
            return listOf("install.txt shell command empty")
        }

        val warnings = mutableListOf<String>()

        when {
            tokens.take(2) == listOf("mkdir", "-p") -> {
                val path = tokens.getOrNull(2)
                if (path == null || !isAllowedShellPath(path, packageName)) {
                    warnings += "install.txt blocked mkdir path: ${path ?: "<missing>"}"
                } else {
                    File(path).mkdirs()
                }
            }

            tokens.take(2) == listOf("rm", "-rf") -> {
                val path = tokens.getOrNull(2)
                if (path == null || !isAllowedShellPath(path, packageName)) {
                    warnings += "install.txt blocked rm path: ${path ?: "<missing>"}"
                } else {
                    File(path).deleteRecursively()
                }
            }

            tokens.take(2) == listOf("pm", "grant") && tokens.size >= 4 -> {
                // `pm grant` is allowed by policy but not directly executable from app process.
                warnings += "install.txt pm grant requires shell privileges; skipped"
                logService.append(operationId, "install", OperationLogLevel.WARN, "Skipped pm grant", command)
            }

            else -> warnings += "install.txt unsupported shell command: $command"
        }

        return warnings
    }

    private fun isAllowedShellPath(path: String, packageName: String): Boolean {
        val normalized = path.trim('"')
        val obbPath = "/sdcard/Android/obb/$packageName"
        val dataPath = "/sdcard/Android/data/$packageName"
        return normalized.startsWith(obbPath) || normalized.startsWith(dataPath)
    }

    private fun copyRecursively(source: File, destination: File) {
        if (source.isDirectory) {
            destination.mkdirs()
            source.listFiles()?.forEach { child ->
                copyRecursively(child, File(destination, child.name))
            }
        } else {
            destination.parentFile?.mkdirs()
            source.copyTo(destination, overwrite = true)
        }
    }
}
