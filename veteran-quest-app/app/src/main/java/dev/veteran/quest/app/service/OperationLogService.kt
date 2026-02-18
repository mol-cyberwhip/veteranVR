package dev.veteran.quest.app.service

import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.OperationLogLevel
import kotlinx.coroutines.flow.StateFlow

interface OperationLogService {
    val logs: StateFlow<List<OperationLogEntry>>

    suspend fun append(
        operationId: String,
        stage: String,
        level: OperationLogLevel,
        message: String,
        details: String? = null,
    )
}
