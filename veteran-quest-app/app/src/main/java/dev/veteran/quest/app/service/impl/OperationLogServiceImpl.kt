package dev.veteran.quest.app.service.impl

import android.content.Context
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.OperationLogLevel
import dev.veteran.quest.app.service.OperationLogService
import dev.veteran.quest.app.util.AppPaths
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import java.io.File

class OperationLogServiceImpl(
    private val context: Context,
    private val maxEntries: Int = 2000,
) : OperationLogService {
    private val json = Json { ignoreUnknownKeys = true }
    private val lock = Mutex()
    private val _logs = MutableStateFlow<List<OperationLogEntry>>(emptyList())

    override val logs: StateFlow<List<OperationLogEntry>> = _logs.asStateFlow()

    init {
        loadExistingLogs()
    }

    override suspend fun append(
        operationId: String,
        stage: String,
        level: OperationLogLevel,
        message: String,
        details: String?,
    ) {
        val entry = OperationLogEntry(
            timestampMs = System.currentTimeMillis(),
            operationId = operationId,
            stage = stage,
            level = level.name,
            message = message,
            details = details,
        )

        lock.withLock {
            val next = (_logs.value + entry).takeLast(maxEntries)
            _logs.value = next
            persist(next)
        }
    }

    private fun loadExistingLogs() {
        val file = AppPaths.logsFile(context)
        if (!file.exists()) {
            return
        }

        val entries = file.readLines()
            .mapNotNull { line ->
                runCatching { json.decodeFromString(OperationLogEntry.serializer(), line) }.getOrNull()
            }
            .takeLast(maxEntries)

        _logs.value = entries
    }

    private suspend fun persist(entries: List<OperationLogEntry>) = withContext(Dispatchers.IO) {
        val file = AppPaths.logsFile(context)
        file.parentFile?.mkdirs()
        file.writeText(entries.joinToString(separator = "\n") { json.encodeToString(OperationLogEntry.serializer(), it) })
    }
}
