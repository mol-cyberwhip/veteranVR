package dev.veteran.quest.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import dev.veteran.quest.app.data.BootstrapQuestRepository
import dev.veteran.quest.app.data.QuestRepository
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.model.SortBy
import dev.veteran.quest.installer.UninstallOptions
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class QuestUiState(
    val loading: Boolean = false,
    val search: String = "",
    val sortBy: SortBy = SortBy.POPULARITY,
    val sortAscending: Boolean = false,
    val filter: LibraryFilter = LibraryFilter.NON_MODS,
    val games: List<Game> = emptyList(),
    val message: String? = null,
    val keepObbOnUninstall: Boolean = false,
    val keepDataOnUninstall: Boolean = false,
)

class QuestViewModel(
    private val repository: QuestRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(QuestUiState())
    val state: StateFlow<QuestUiState> = _state.asStateFlow()

    init {
        refreshCatalog(force = false)
    }

    fun onSearchChanged(value: String) {
        _state.value = _state.value.copy(search = value)
        reloadLibrary()
    }

    fun onSortByChanged(sortBy: SortBy) {
        _state.value = _state.value.copy(sortBy = sortBy)
        reloadLibrary()
    }

    fun onSortDirectionToggled() {
        _state.value = _state.value.copy(sortAscending = !_state.value.sortAscending)
        reloadLibrary()
    }

    fun onFilterChanged(filter: LibraryFilter) {
        _state.value = _state.value.copy(filter = filter)
        reloadLibrary()
    }

    fun onKeepObbChanged(checked: Boolean) {
        _state.value = _state.value.copy(keepObbOnUninstall = checked)
    }

    fun onKeepDataChanged(checked: Boolean) {
        _state.value = _state.value.copy(keepDataOnUninstall = checked)
    }

    fun refreshCatalog(force: Boolean) {
        viewModelScope.launch {
            _state.value = _state.value.copy(loading = true, message = null)
            repository.syncCatalog(force)
                .onSuccess { count ->
                    _state.value = _state.value.copy(message = "Catalog ready ($count titles)")
                }
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Sync failed: ${err.message}")
                }
            reloadLibrary()
        }
    }

    fun install(game: Game) {
        viewModelScope.launch {
            repository.install(game)
                .onSuccess {
                    _state.value = _state.value.copy(message = "Install started: ${game.gameName}")
                }
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Install failed: ${err.message}")
                }
        }
    }

    fun uninstall(game: Game) {
        viewModelScope.launch {
            repository.uninstall(
                packageName = game.packageName,
                options = UninstallOptions(
                    keepObb = _state.value.keepObbOnUninstall,
                    keepData = _state.value.keepDataOnUninstall,
                ),
            ).onSuccess {
                _state.value = _state.value.copy(message = "Uninstalled: ${game.gameName}")
            }.onFailure { err ->
                _state.value = _state.value.copy(message = "Uninstall failed: ${err.message}")
            }
        }
    }

    private fun reloadLibrary() {
        viewModelScope.launch {
            val current = _state.value
            val query = LibraryQuery(
                search = current.search,
                sortBy = current.sortBy,
                sortAscending = current.sortAscending,
                filter = current.filter,
            )
            val games = repository.getLibrary(query)
            _state.value = _state.value.copy(loading = false, games = games)
        }
    }

    companion object {
        val Factory = object : ViewModelProvider.Factory {
            override fun <T : ViewModel> create(modelClass: Class<T>): T {
                @Suppress("UNCHECKED_CAST")
                return QuestViewModel(BootstrapQuestRepository()) as T
            }
        }
    }
}
