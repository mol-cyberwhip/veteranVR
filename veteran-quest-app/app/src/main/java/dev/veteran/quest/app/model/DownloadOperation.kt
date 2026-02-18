package dev.veteran.quest.app.model

enum class DownloadState {
    QUEUED,
    DOWNLOADING,
    PAUSED,
    EXTRACTING,
    INSTALLING,
    COMPLETED,
    FAILED,
    CANCELLED,
}

data class DownloadOperation(
    val operationId: String,
    val packageName: String,
    val releaseName: String,
    val state: DownloadState,
    val progressPercent: Double,
    val bytesDone: Long,
    val bytesTotal: Long,
    val speedBps: Long,
    val etaSeconds: Long,
    val message: String = "",
)
