package dev.veteran.quest.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import dev.veteran.quest.app.AppContainer
import dev.veteran.quest.app.data.QuestRepository
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.PermissionGateStatus
import dev.veteran.quest.app.ui.tokens.MotionProfile
import dev.veteran.quest.app.ui.tokens.UiDensity
import dev.veteran.quest.installer.UninstallOptions
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.LibraryQuery
import dev.veteran.quest.model.SortBy
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class QuestUiState(
    val loading: Boolean = false,
    val syncing: Boolean = false,
    val search: String = "",
    val sortBy: SortBy = SortBy.POPULARITY,
    val sortAscending: Boolean = false,
    val filter: LibraryFilter = LibraryFilter.NON_MODS,
    val games: List<LibraryItemUi> = emptyList(),
    val operations: List<DownloadOperation> = emptyList(),
    val logs: List<OperationLogEntry> = emptyList(),
    val message: String? = null,
    val keepObbOnUninstall: Boolean = false,
    val keepDataOnUninstall: Boolean = false,
    val keepAwakeDuringOps: Boolean = true,
    val permissionStatus: PermissionGateStatus? = null,
    val showDiagnostics: Boolean = false,
    val uiDensity: UiDensity = UiDensity.COMFORTABLE,
    val motionProfile: MotionProfile = MotionProfile.SUBTLE,
) {
    val activeOperation: DownloadOperation?
        get() = operations.firstOrNull {
            it.state in listOf(
                DownloadState.QUEUED,
                DownloadState.DOWNLOADING,
                DownloadState.EXTRACTING,
                DownloadState.INSTALLING,
            )
        }

    val activeOperationLabel: String?
        get() = activeOperation?.let { op ->
            "${op.releaseName} ${op.state.name.lowercase()} ${op.progressPercent.toInt()}%"
        }
}

class QuestViewModel(
    private val repository: QuestRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(QuestUiState())
    val state: StateFlow<QuestUiState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            launch {
                repository.operations.collect { ops ->
                    _state.value = _state.value.copy(operations = ops)
                }
            }
            launch {
                repository.logs.collect { logs ->
                    _state.value = _state.value.copy(logs = logs.takeLast(500).reversed())
                }
            }
        }

        refreshPermissionStatus()
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

    fun onKeepAwakeChanged(checked: Boolean) {
        _state.value = _state.value.copy(keepAwakeDuringOps = checked)
    }

    fun onShowDiagnosticsChanged(show: Boolean) {
        _state.value = _state.value.copy(showDiagnostics = show)
    }

    fun onUiDensityChanged(density: UiDensity) {
        _state.value = _state.value.copy(uiDensity = density)
    }

    fun onMotionProfileChanged(profile: MotionProfile) {
        _state.value = _state.value.copy(motionProfile = profile)
    }

    fun refreshCatalog(force: Boolean) {
        viewModelScope.launch {
            _state.value = _state.value.copy(syncing = true, message = null)
            repository.syncCatalog(force)
                .onSuccess { summary ->
                    val source = if (summary.usedCache) "cache" else "remote"
                    _state.value = _state.value.copy(
                        message = "Catalog ready (${summary.gamesCount} titles, $source)",
                    )
                }
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Sync failed: ${err.message}")
                }
            _state.value = _state.value.copy(syncing = false)
            reloadLibrary()
            refreshPermissionStatus()
        }
    }

    fun enqueueInstall(game: Game) {
        viewModelScope.launch {
            val setupReady = _state.value.permissionStatus?.isReady == true
            if (!setupReady) {
                _state.value = _state.value.copy(message = "Complete setup gate before installing")
                return@launch
            }
            repository.enqueueInstall(game)
                .onSuccess { opId ->
                    _state.value = _state.value.copy(message = "Queued install: ${game.gameName} ($opId)")
                }
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Queue failed: ${err.message}")
                }
        }
    }

    fun pauseOperation(operationId: String) {
        viewModelScope.launch {
            repository.pauseDownload(operationId)
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Pause failed: ${err.message}")
                }
        }
    }

    fun resumeOperation(operationId: String) {
        viewModelScope.launch {
            repository.resumeDownload(operationId)
                .onFailure { err ->
                    _state.value = _state.value.copy(message = "Resume failed: ${err.message}")
                }
        }
    }

    fun uninstall(game: Game) {
        viewModelScope.launch {
            val opId = "uninstall-${game.packageName}"
            _state.value = _state.value.copy(message = "Uninstalling ${game.gameName}...")
            repository.uninstall(
                packageName = game.packageName,
                options = UninstallOptions(
                    keepObb = _state.value.keepObbOnUninstall,
                    keepData = _state.value.keepDataOnUninstall,
                ),
            ).onSuccess {
                _state.value = _state.value.copy(message = "Uninstalled: ${game.gameName}")
                refreshCatalog(force = false)
            }.onFailure { err ->
                _state.value = _state.value.copy(message = "Uninstall failed ($opId): ${err.message}")
            }
        }
    }

    fun refreshPermissionStatus() {
        viewModelScope.launch {
            val status = repository.permissionStatus()
            _state.value = _state.value.copy(permissionStatus = status)
        }
    }

    private fun reloadLibrary() {
        viewModelScope.launch {
            _state.value = _state.value.copy(loading = true)
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
        fun factory(container: AppContainer): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    @Suppress("UNCHECKED_CAST")
                    return QuestViewModel(
                        repository = container.repository,
                    ) as T
                }
            }
        }
    }
}
