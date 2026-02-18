package dev.veteran.quest.app.model

data class CatalogSyncSummary(
    val lastSyncEpochMs: Long,
    val gamesCount: Int,
    val usedCache: Boolean,
)
