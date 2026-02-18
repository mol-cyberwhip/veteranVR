package dev.veteran.quest.catalog

import dev.veteran.quest.model.Game

data class CatalogDataset(
    val latestGames: List<Game>,
    val allVersions: List<Game>,
)
