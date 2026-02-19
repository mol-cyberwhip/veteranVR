package dev.veteran.quest.app.service.impl

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.Settings
import dev.veteran.quest.app.model.PermissionGateStatus
import dev.veteran.quest.app.service.PermissionGateService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

class PermissionGateServiceImpl(
    private val context: Context,
    private val minRequiredBytes: Long = 5L * 1024L * 1024L * 1024L,
) : PermissionGateService {
    override suspend fun evaluate(): PermissionGateStatus = withContext(Dispatchers.IO) {
        PermissionGateStatus(
            canInstallPackages = context.packageManager.canRequestPackageInstalls(),
            hasAllFilesAccess = hasAllFilesAccess(),
            freeBytes = context.filesDir.freeSpace,
            minRequiredBytes = minRequiredBytes,
        )
    }

    override fun openInstallPermissionSettingsIntent(): Intent {
        return Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES).apply {
            data = Uri.parse("package:${context.packageName}")
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
    }

    override fun openAllFilesPermissionSettingsIntent(): Intent {
        val action = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION
        } else {
            Settings.ACTION_APPLICATION_DETAILS_SETTINGS
        }

        return Intent(action).apply {
            data = Uri.parse("package:${context.packageName}")
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
    }

    private fun hasAllFilesAccess(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Environment.isExternalStorageManager()
        } else {
            // targetSdk 29 fallback
            File(Environment.getExternalStorageDirectory(), "Android").canRead()
        }
    }
}
