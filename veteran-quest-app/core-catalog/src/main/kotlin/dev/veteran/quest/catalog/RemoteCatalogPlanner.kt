package dev.veteran.quest.catalog

data class RemoteChunkFile(
    val name: String,
    val url: String,
)

object RemoteCatalogPlanner {
    fun metaArchiveUrl(baseUri: String): String {
        val base = baseUri.trimEnd('/')
        return "$base/meta.7z"
    }

    fun gameChunkUrls(baseUri: String, gameHash: String, hashIndexHtml: String): List<RemoteChunkFile> {
        val base = baseUri.trimEnd('/')
        return ApacheIndexParser.parseEntries(hashIndexHtml)
            .filter { name ->
                val lower = name.lowercase()
                lower.startsWith(gameHash.lowercase()) && (lower.endsWith(".7z") || lower.contains(".7z."))
            }
            .sortedWith(compareBy< String > { !it.lowercase().endsWith(".7z.001") }.thenBy { it })
            .map { name ->
                RemoteChunkFile(
                    name = name,
                    url = "$base/$gameHash/$name",
                )
            }
    }
}
