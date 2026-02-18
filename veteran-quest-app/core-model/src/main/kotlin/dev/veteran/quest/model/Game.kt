package dev.veteran.quest.model

import kotlinx.serialization.Serializable

@Serializable
data class Game(
    val gameName: String,
    val releaseName: String,
    val packageName: String,
    val versionCode: String,
    val releaseApkPath: String = "",
    val versionName: String = "",
    val downloads: String = "",
    val size: String = "",
    val lastUpdated: String = "",
    val thumbnailPath: String = "",
    val thumbnailExists: Boolean = false,
    val notePath: String = "",
    val noteExcerpt: String = "",
    val noteExists: Boolean = false,
    val popularityRank: Int = 0,
    val isNew: Boolean = false,
) {
    val isModded: Boolean
        get() {
            val lowerName = "$gameName $releaseName".lowercase()
            return "-mod" in lowerName || " mod " in lowerName || "modded" in lowerName
        }
}
