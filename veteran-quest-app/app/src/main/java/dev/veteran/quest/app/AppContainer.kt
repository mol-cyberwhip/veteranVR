package dev.veteran.quest.app

import android.content.Context
import dev.veteran.quest.app.data.QuestRemoteDataSource
import dev.veteran.quest.app.data.QuestRepository
import dev.veteran.quest.app.data.QuestRepositoryImpl
import dev.veteran.quest.app.platform.PackageInstallerBridge
import dev.veteran.quest.app.service.PermissionGateService
import dev.veteran.quest.app.service.impl.CatalogSyncServiceImpl
import dev.veteran.quest.app.service.impl.DownloadQueueServiceImpl
import dev.veteran.quest.app.service.impl.ExtractionServiceImpl
import dev.veteran.quest.app.service.impl.OperationLogServiceImpl
import dev.veteran.quest.app.service.impl.PackageInstallServiceImpl
import dev.veteran.quest.app.service.impl.PermissionGateServiceImpl

class AppContainer private constructor(context: Context) {
    private val appContext = context.applicationContext

    private val remoteDataSource = QuestRemoteDataSource()
    private val logService = OperationLogServiceImpl(appContext)
    private val extractionService = ExtractionServiceImpl(appContext)
    private val packageBridge = PackageInstallerBridge(appContext)
    private val permissionGateServiceImpl = PermissionGateServiceImpl(appContext)
    private val packageInstallService = PackageInstallServiceImpl(appContext, packageBridge, logService)
    private val catalogSyncService = CatalogSyncServiceImpl(appContext, remoteDataSource, extractionService, logService)
    private val downloadQueueService = DownloadQueueServiceImpl(
        context = appContext,
        remote = remoteDataSource,
        extraction = extractionService,
        packageInstall = packageInstallService,
        logService = logService,
        configProvider = { remoteDataSource.fetchPublicConfig() },
    )

    val repository: QuestRepository = QuestRepositoryImpl(
        context = appContext,
        catalogSyncService = catalogSyncService,
        downloadQueueService = downloadQueueService,
        packageInstallService = packageInstallService,
        logService = logService,
        permissionGateService = permissionGateServiceImpl,
    )

    val permissionGateService: PermissionGateService = permissionGateServiceImpl

    companion object {
        @Volatile
        private var instance: AppContainer? = null

        fun from(context: Context): AppContainer {
            return instance ?: synchronized(this) {
                instance ?: AppContainer(context).also { instance = it }
            }
        }
    }
}
