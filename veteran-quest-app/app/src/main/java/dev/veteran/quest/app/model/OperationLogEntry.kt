package dev.veteran.quest.app.model

import kotlinx.serialization.Serializable

enum class OperationLogLevel {
    INFO,
    WARN,
    ERROR,
}

@Serializable
data class OperationLogEntry(
    val timestampMs: Long,
    val operationId: String,
    val stage: String,
    val level: String,
    val message: String,
    val details: String? = null,
)
