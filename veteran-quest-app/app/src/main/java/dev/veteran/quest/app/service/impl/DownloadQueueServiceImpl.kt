package dev.veteran.quest.app.service.impl

import android.content.Context
import dev.veteran.quest.app.data.QuestRemoteDataSource
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.OperationLogLevel
import dev.veteran.quest.app.platform.DownloadForegroundService
import dev.veteran.quest.app.service.DownloadQueueService
import dev.veteran.quest.app.service.ExtractionService
import dev.veteran.quest.app.service.OperationLogService
import dev.veteran.quest.app.service.PackageInstallService
import dev.veteran.quest.app.util.AppPaths
import dev.veteran.quest.catalog.CatalogHash
import dev.veteran.quest.catalog.RemoteCatalogPlanner
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.PublicConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.io.File
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.roundToInt

class DownloadQueueServiceImpl(
    private val context: Context,
    private val remote: QuestRemoteDataSource,
    private val extraction: ExtractionService,
    private val packageInstall: PackageInstallService,
    private val logService: OperationLogService,
    private val configProvider: suspend () -> PublicConfig,
) : DownloadQueueService {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val lock = Mutex()
    private val pauseRequested = ConcurrentHashMap<String, Boolean>()
    private val operationGames = ConcurrentHashMap<String, Game>()

    private val _operations = MutableStateFlow<List<DownloadOperation>>(emptyList())
    override val operations: StateFlow<List<DownloadOperation>> = _operations.asStateFlow()

    init {
        scope.launch {
            workerLoop()
        }
    }

    override suspend fun enqueueInstall(game: Game): Result<String> {
        return runCatching {
            lock.withLock {
                val existing = _operations.value.firstOrNull {
                    it.packageName == game.packageName && it.state !in listOf(DownloadState.FAILED, DownloadState.CANCELLED, DownloadState.COMPLETED)
                }
                if (existing != null) {
                    return@withLock existing.operationId
                }

                val operationId = UUID.randomUUID().toString()
                operationGames[operationId] = game
                _operations.value = _operations.value + DownloadOperation(
                    operationId = operationId,
                    packageName = game.packageName,
                    releaseName = game.releaseName,
                    state = DownloadState.QUEUED,
                    progressPercent = 0.0,
                    bytesDone = 0,
                    bytesTotal = 0,
                    speedBps = 0,
                    etaSeconds = 0,
                    message = "Queued",
                )
                operationId
            }
        }
    }

    override suspend fun pause(operationId: String): Result<Unit> {
        pauseRequested[operationId] = true
        updateState(operationId) { current ->
            if (current.state == DownloadState.DOWNLOADING) {
                current.copy(state = DownloadState.PAUSED, message = "Pause requested")
            } else {
                current
            }
        }
        return Result.success(Unit)
    }

    override suspend fun resume(operationId: String): Result<Unit> {
        pauseRequested[operationId] = false
        updateState(operationId) { current ->
            if (current.state == DownloadState.PAUSED || current.state == DownloadState.FAILED) {
                current.copy(state = DownloadState.QUEUED, message = "Queued")
            } else {
                current
            }
        }
        return Result.success(Unit)
    }

    private suspend fun workerLoop() {
        while (true) {
            val next = lock.withLock {
                _operations.value.firstOrNull { it.state == DownloadState.QUEUED }
            }

            if (next == null) {
                maybeStopForegroundService()
                delay(250)
                continue
            }

            val game = operationGames[next.operationId]
            if (game == null) {
                updateState(next.operationId) { it.copy(state = DownloadState.FAILED, message = "Missing game context") }
                continue
            }

            DownloadForegroundService.start(context, "Downloading ${game.gameName}")

            try {
                processOperation(next.operationId, game)
                updateState(next.operationId) { it.copy(state = DownloadState.COMPLETED, progressPercent = 100.0, message = "Installed") }
                logService.append(next.operationId, "install", OperationLogLevel.INFO, "Install completed", game.packageName)
            } catch (pause: PauseRequestedException) {
                updateState(next.operationId) { it.copy(state = DownloadState.PAUSED, message = "Paused") }
                logService.append(next.operationId, "download", OperationLogLevel.INFO, "Download paused")
            } catch (t: Throwable) {
                updateState(next.operationId) { it.copy(state = DownloadState.FAILED, message = t.message ?: "Failed") }
                logService.append(next.operationId, "download", OperationLogLevel.ERROR, "Operation failed", t.message)
            }
        }
    }

    private suspend fun processOperation(operationId: String, game: Game) {
        val config = configProvider()
        val hash = CatalogHash.gameNameToHash(game.releaseName)
        val base = config.baseUri.trimEnd('/')
        val hashDir = File(AppPaths.downloadsRoot(context), hash).apply { mkdirs() }
        val indexUrl = "$base/$hash/"

        logService.append(operationId, "download", OperationLogLevel.INFO, "Fetching remote chunk list", indexUrl)
        val indexHtml = remote.fetchText(indexUrl)
        val chunks = RemoteCatalogPlanner.gameChunkUrls(base, hash, indexHtml)
        check(chunks.isNotEmpty()) { "No archive chunks found for ${game.releaseName}" }

        val headResults = chunks.associate { chunk -> chunk.name to remote.head(chunk.url) }
        val totalBytes = headResults.values.sumOf { if (it.contentLength > 0) it.contentLength else 0L }
        var doneBytes = chunks.sumOf { chunk -> File(hashDir, chunk.name).takeIf { it.exists() }?.length() ?: 0L }
        var lastEmitTime = System.currentTimeMillis()
        var lastEmitBytes = doneBytes

        updateState(operationId) {
            it.copy(
                state = DownloadState.DOWNLOADING,
                bytesTotal = totalBytes,
                bytesDone = doneBytes,
                progressPercent = if (totalBytes <= 0) 0.0 else (doneBytes.toDouble() / totalBytes.toDouble()) * 100.0,
                message = "Downloading",
            )
        }

        for (chunk in chunks) {
            ensureNotPaused(operationId)
            val destination = File(hashDir, chunk.name)
            val head = headResults[chunk.name] ?: continue

            if (destination.exists() && head.contentLength > 0 && destination.length() == head.contentLength) {
                continue
            }

            if (destination.exists() && !head.acceptRanges) {
                destination.delete()
                logService.append(operationId, "download", OperationLogLevel.WARN, "Range unsupported, restarting chunk", chunk.name)
            }

            remote.downloadToFile(
                url = chunk.url,
                destination = destination,
                resume = head.acceptRanges,
            ) { read ->
                if (pauseRequested[operationId] == true) {
                    throw PauseRequestedException()
                }
                doneBytes += read
                val now = System.currentTimeMillis()
                if (now - lastEmitTime >= 500) {
                    val elapsedMs = (now - lastEmitTime).coerceAtLeast(1)
                    val bytesWindow = (doneBytes - lastEmitBytes).coerceAtLeast(0)
                    val speedBps = (bytesWindow * 1000L) / elapsedMs
                    val remaining = (totalBytes - doneBytes).coerceAtLeast(0)
                    val etaSeconds = if (speedBps > 0) remaining / speedBps else 0L
                    val percent = if (totalBytes <= 0L) 0.0 else (doneBytes.toDouble() / totalBytes.toDouble()) * 100.0

                    updateStateSync(operationId) {
                        it.copy(
                            state = DownloadState.DOWNLOADING,
                            bytesDone = doneBytes,
                            bytesTotal = totalBytes,
                            speedBps = speedBps,
                            etaSeconds = etaSeconds,
                            progressPercent = percent,
                            message = "Downloading ${percent.roundToInt()}%",
                        )
                    }

                    DownloadForegroundService.update(context, "${game.gameName} ${percent.roundToInt()}%")
                    lastEmitTime = now
                    lastEmitBytes = doneBytes
                }
            }
        }

        updateState(operationId) { it.copy(state = DownloadState.EXTRACTING, message = "Extracting") }
        logService.append(operationId, "extract", OperationLogLevel.INFO, "Extracting game archive", hash)

        val archiveStart = chunks
            .map { File(hashDir, it.name) }
            .sortedBy { it.name }
            .firstOrNull { it.name.lowercase().endsWith(".7z.001") }
            ?: chunks.map { File(hashDir, it.name) }.sortedBy { it.name }.first()

        val downloadRoot = AppPaths.downloadsRoot(context).apply { mkdirs() }
        extraction.extract7z(archiveStart, downloadRoot, config.password).getOrThrow()

        val gameDir = File(downloadRoot, game.releaseName).takeIf { it.exists() } ?: hashDir

        updateState(operationId) { it.copy(state = DownloadState.INSTALLING, message = "Installing") }
        val report = packageInstall.installFromExtractedGameDir(operationId, gameDir, game.packageName).getOrThrow()
        if (report.warnings.isNotEmpty()) {
            logService.append(operationId, "install", OperationLogLevel.WARN, "Install warnings", report.warnings.joinToString("; "))
        }

        hashDir.deleteRecursively()
        if (gameDir.exists()) {
            gameDir.deleteRecursively()
        }
    }

    private suspend fun maybeStopForegroundService() {
        val active = _operations.value.any { op ->
            op.state in listOf(DownloadState.QUEUED, DownloadState.DOWNLOADING, DownloadState.EXTRACTING, DownloadState.INSTALLING)
        }

        if (!active) {
            DownloadForegroundService.stop(context)
        }
    }

    private fun ensureNotPaused(operationId: String) {
        if (pauseRequested[operationId] == true) {
            throw PauseRequestedException()
        }
    }

    private suspend fun updateState(operationId: String, mutate: (DownloadOperation) -> DownloadOperation) {
        lock.withLock {
            _operations.value = _operations.value.map { op ->
                if (op.operationId == operationId) mutate(op) else op
            }
        }
    }

    private fun updateStateSync(operationId: String, mutate: (DownloadOperation) -> DownloadOperation) {
        _operations.value = _operations.value.map { op ->
            if (op.operationId == operationId) mutate(op) else op
        }
    }

    private class PauseRequestedException : RuntimeException()
}
