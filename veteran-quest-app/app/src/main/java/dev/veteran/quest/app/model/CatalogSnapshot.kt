package dev.veteran.quest.app.model

import dev.veteran.quest.catalog.CatalogDataset
import dev.veteran.quest.model.PublicConfig

data class CatalogSnapshot(
    val dataset: CatalogDataset,
    val summary: CatalogSyncSummary,
    val config: PublicConfig,
)
