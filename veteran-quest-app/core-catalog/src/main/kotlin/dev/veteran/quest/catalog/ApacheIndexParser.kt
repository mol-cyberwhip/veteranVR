package dev.veteran.quest.catalog

object ApacheIndexParser {
    private val hrefRegex = Regex("""href=\"([^\"]+)\"""")

    fun parseEntries(indexHtml: String): List<String> {
        return hrefRegex.findAll(indexHtml)
            .map { it.groupValues[1] }
            .filter { href -> href.isNotBlank() && href != "../" }
            .map { it.trim() }
            .toList()
    }
}
