package dev.veteran.quest.model

enum class SortBy {
    NAME,
    DATE,
    SIZE,
    POPULARITY,
}

enum class LibraryFilter {
    ALL,
    FAVORITES,
    NEW,
    POPULAR,
    NON_MODS,
}

data class LibraryQuery(
    val search: String = "",
    val sortBy: SortBy = SortBy.POPULARITY,
    val sortAscending: Boolean = false,
    val filter: LibraryFilter = LibraryFilter.ALL,
    val favorites: Set<String> = emptySet(),
)
