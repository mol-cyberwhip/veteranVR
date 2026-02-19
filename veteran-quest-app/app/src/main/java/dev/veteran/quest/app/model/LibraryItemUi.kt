package dev.veteran.quest.app.model

import dev.veteran.quest.model.Game

data class LibraryItemUi(
    val game: Game,
    val thumbnailPath: String,
    val thumbnailExists: Boolean,
    val notePath: String,
    val noteExists: Boolean,
    val isInstalled: Boolean,
)
