package dev.veteran.quest.app.ui

import com.google.common.truth.Truth.assertThat
import dev.veteran.quest.app.data.QuestRepository
import dev.veteran.quest.app.model.CatalogSyncSummary
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.model.PermissionGateStatus
import dev.veteran.quest.app.ui.tokens.UiDensity
import dev.veteran.quest.installer.UninstallOptions
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryQuery
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class QuestViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `refreshCatalog updates message when sync succeeds`() = runTest {
        val repo = FakeRepository()
        val vm = QuestViewModel(repo)

        vm.refreshCatalog(force = true)
        advanceUntilIdle()

        assertThat(vm.state.value.message).contains("Catalog ready")
    }

    @Test
    fun `enqueueInstall blocked when setup gate not ready`() = runTest {
        val repo = FakeRepository(
            permission = PermissionGateStatus(
                canInstallPackages = false,
                hasAllFilesAccess = false,
                freeBytes = 0,
                minRequiredBytes = 10,
            ),
        )
        val vm = QuestViewModel(repo)

        advanceUntilIdle()
        vm.enqueueInstall(
            Game(
                gameName = "Sample",
                releaseName = "Sample v1",
                packageName = "pkg.sample",
                versionCode = "1",
            ),
        )
        advanceUntilIdle()

        assertThat(vm.state.value.message).contains("Complete setup gate")
    }

    @Test
    fun `active operation label formats status and percent`() = runTest {
        val repo = FakeRepository()
        val vm = QuestViewModel(repo)

        repo.emitOperation(
            DownloadOperation(
                operationId = "op-1",
                packageName = "pkg.sample",
                releaseName = "Sample v1",
                state = DownloadState.DOWNLOADING,
                progressPercent = 42.2,
                bytesDone = 10,
                bytesTotal = 20,
                speedBps = 123,
                etaSeconds = 8,
            ),
        )
        advanceUntilIdle()

        assertThat(vm.state.value.activeOperationLabel).isEqualTo("Sample v1 downloading 42%")
    }

    @Test
    fun `diagnostics visibility and density can be toggled`() = runTest {
        val repo = FakeRepository()
        val vm = QuestViewModel(repo)

        vm.onShowDiagnosticsChanged(true)
        vm.onUiDensityChanged(UiDensity.BALANCED)
        advanceUntilIdle()

        assertThat(vm.state.value.showDiagnostics).isTrue()
        assertThat(vm.state.value.uiDensity).isEqualTo(UiDensity.BALANCED)
    }
}

private class FakeRepository(
    private val permission: PermissionGateStatus = PermissionGateStatus(true, true, 100, 10),
) : QuestRepository {
    private val operationFlow = MutableStateFlow<List<DownloadOperation>>(emptyList())
    private val logFlow = MutableStateFlow<List<OperationLogEntry>>(emptyList())

    override val operations: StateFlow<List<DownloadOperation>> = operationFlow
    override val logs: StateFlow<List<OperationLogEntry>> = logFlow

    fun emitOperation(operation: DownloadOperation) {
        operationFlow.value = listOf(operation)
    }

    override suspend fun syncCatalog(force: Boolean): Result<CatalogSyncSummary> {
        return Result.success(CatalogSyncSummary(System.currentTimeMillis(), 4, false))
    }

    override suspend fun getLibrary(query: LibraryQuery): List<LibraryItemUi> {
        return emptyList()
    }

    override suspend fun enqueueInstall(game: Game): Result<String> {
        return Result.success("op-1")
    }

    override suspend fun pauseDownload(operationId: String): Result<Unit> = Result.success(Unit)
    override suspend fun resumeDownload(operationId: String): Result<Unit> = Result.success(Unit)

    override suspend fun uninstall(packageName: String, options: UninstallOptions): Result<Unit> =
        Result.success(Unit)

    override suspend fun permissionStatus(): PermissionGateStatus = permission
}
