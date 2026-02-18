package dev.veteran.quest.app.model

data class PermissionGateStatus(
    val canInstallPackages: Boolean,
    val hasAllFilesAccess: Boolean,
    val freeBytes: Long,
    val minRequiredBytes: Long,
) {
    val isReady: Boolean
        get() = canInstallPackages && hasAllFilesAccess && freeBytes >= minRequiredBytes
}
