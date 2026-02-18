package dev.veteran.quest.app.service

import android.content.Intent
import dev.veteran.quest.app.model.PermissionGateStatus

interface PermissionGateService {
    suspend fun evaluate(): PermissionGateStatus
    fun openInstallPermissionSettingsIntent(): Intent
    fun openAllFilesPermissionSettingsIntent(): Intent
}
