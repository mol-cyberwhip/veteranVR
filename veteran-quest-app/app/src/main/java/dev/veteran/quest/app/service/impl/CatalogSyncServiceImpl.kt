package dev.veteran.quest.app.service.impl

import android.content.Context
import dev.veteran.quest.app.data.QuestRemoteDataSource
import dev.veteran.quest.app.model.CatalogSnapshot
import dev.veteran.quest.app.model.CatalogSyncSummary
import dev.veteran.quest.app.model.OperationLogLevel
import dev.veteran.quest.app.service.CatalogSyncService
import dev.veteran.quest.app.service.ExtractionService
import dev.veteran.quest.app.service.OperationLogService
import dev.veteran.quest.app.util.AppPaths
import dev.veteran.quest.catalog.CatalogParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption

class CatalogSyncServiceImpl(
    private val context: Context,
    private val remote: QuestRemoteDataSource,
    private val extraction: ExtractionService,
    private val logService: OperationLogService,
) : CatalogSyncService {

    override suspend fun syncCatalog(force: Boolean): Result<CatalogSnapshot> = withContext(Dispatchers.IO) {
        runCatching {
            val opId = "catalog-sync"
            val cacheCatalog = AppPaths.cacheCatalogFile(context)
            cacheCatalog.parentFile?.mkdirs()

            val fresh = cacheCatalog.exists() && isFresh(cacheCatalog, hours = 4)
            if (fresh && !force) {
                val cached = cacheCatalog.readText()
                val dataset = CatalogParser.parseGameList(cached)
                val config = remote.fetchPublicConfig()
                logService.append(opId, "sync", OperationLogLevel.INFO, "Using cached catalog (<4h)")
                return@runCatching CatalogSnapshot(
                    dataset = dataset,
                    summary = CatalogSyncSummary(
                        lastSyncEpochMs = cacheCatalog.lastModified(),
                        gamesCount = dataset.latestGames.size,
                        usedCache = true,
                    ),
                    config = config,
                )
            }

            logService.append(opId, "sync", OperationLogLevel.INFO, "Fetching public config")
            val config = remote.fetchPublicConfig()
            val metaUrl = "${config.baseUri.trimEnd('/')}/meta.7z"

            val metaDownloadDir = AppPaths.cacheMetaDownloadDir(context).apply { mkdirs() }
            val metaArchive = File(metaDownloadDir, "meta.7z")
            val oldModified = if (metaArchive.exists()) metaArchive.lastModified() else -1L

            logService.append(opId, "sync", OperationLogLevel.INFO, "Downloading meta.7z")
            remote.downloadToFile(
                url = metaUrl,
                destination = metaArchive,
                resume = true,
                onChunk = {},
            )

            val extractDir = AppPaths.cacheMetaExtractedDir(context)
            val shouldExtract = !extractDir.exists() || !cacheCatalog.exists() || metaArchive.lastModified() != oldModified
            if (shouldExtract) {
                logService.append(opId, "extract", OperationLogLevel.INFO, "Extracting meta.7z")
                if (extractDir.exists()) {
                    extractDir.deleteRecursively()
                }
                extractDir.mkdirs()

                extraction.extract7z(metaArchive, extractDir, config.password).getOrElse { error ->
                    throw IllegalStateException("Failed to extract meta.7z", error)
                }
            }

            val gameListPath = listOf(
                File(extractDir, "VRP-GameList.txt"),
                File(File(extractDir, ".meta"), "VRP-GameList.txt"),
            ).firstOrNull { it.exists() }
                ?: throw IllegalStateException("VRP-GameList.txt not found after extraction")

            Files.copy(
                gameListPath.toPath(),
                cacheCatalog.toPath(),
                StandardCopyOption.REPLACE_EXISTING,
            )

            syncMetaAssets(extractDir)

            val content = cacheCatalog.readText()
            val dataset = CatalogParser.parseGameList(content)

            logService.append(opId, "sync", OperationLogLevel.INFO, "Catalog sync complete: ${dataset.latestGames.size} titles")

            CatalogSnapshot(
                dataset = dataset,
                summary = CatalogSyncSummary(
                    lastSyncEpochMs = cacheCatalog.lastModified(),
                    gamesCount = dataset.latestGames.size,
                    usedCache = false,
                ),
                config = config,
            )
        }
    }

    private suspend fun syncMetaAssets(extractDir: File) {
        val extractedMeta = File(extractDir, ".meta")
        if (!extractedMeta.exists()) {
            return
        }

        copyDirectoryContent(
            source = File(extractedMeta, "thumbnails"),
            target = AppPaths.cacheThumbnailsDir(context).apply { mkdirs() },
        )
        copyDirectoryContent(
            source = File(extractedMeta, "notes"),
            target = AppPaths.cacheNotesDir(context).apply { mkdirs() },
        )
    }

    private fun copyDirectoryContent(source: File, target: File) {
        if (!source.isDirectory) {
            return
        }

        source.listFiles()?.forEach { child ->
            if (!child.isFile) {
                return@forEach
            }
            Files.copy(
                child.toPath(),
                File(target, child.name).toPath(),
                StandardCopyOption.REPLACE_EXISTING,
            )
        }
    }

    private fun isFresh(file: File, hours: Long): Boolean {
        val ageMillis = System.currentTimeMillis() - file.lastModified()
        val cutoff = hours * 60 * 60 * 1000
        return ageMillis in 0 until cutoff
    }
}
