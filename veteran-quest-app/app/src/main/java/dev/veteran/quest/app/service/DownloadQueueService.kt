package dev.veteran.quest.app.service

import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.model.Game
import kotlinx.coroutines.flow.StateFlow

interface DownloadQueueService {
    val operations: StateFlow<List<DownloadOperation>>

    suspend fun enqueueInstall(game: Game): Result<String>
    suspend fun pause(operationId: String): Result<Unit>
    suspend fun resume(operationId: String): Result<Unit>
}
