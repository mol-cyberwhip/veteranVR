package dev.veteran.quest.app.util

import android.content.Context
import java.io.File

object AppPaths {
    private const val ROOT_DIR = "veteran"

    fun root(context: Context): File = File(context.filesDir, ROOT_DIR)

    fun cacheRoot(context: Context): File = File(root(context), "cache")
    fun cacheCatalogFile(context: Context): File = File(cacheRoot(context), "VRP-GameList.txt")
    fun cacheMetaDownloadDir(context: Context): File = File(cacheRoot(context), "meta_download")
    fun cacheMetaExtractedDir(context: Context): File = File(cacheRoot(context), "meta_extracted")
    fun cacheThumbnailsDir(context: Context): File = File(cacheRoot(context), "thumbnails")
    fun cacheNotesDir(context: Context): File = File(cacheRoot(context), "notes")
    fun downloadsRoot(context: Context): File = File(root(context), "downloads")
    fun binariesRoot(context: Context): File = File(root(context), "bin")
    fun logsFile(context: Context): File = File(root(context), "operation_logs.jsonl")
}
