package dev.veteran.quest.installer

sealed interface InstallAction {
    data class InstallApk(val relativeApkPath: String) : InstallAction
    data class PushDirectory(val relativeLocalPath: String, val remotePath: String) : InstallAction
    data class Shell(val command: String) : InstallAction
}

data class InstallPlan(
    val actions: List<InstallAction>,
    val warnings: List<String> = emptyList(),
)

data class UninstallOptions(
    val keepObb: Boolean,
    val keepData: Boolean,
)
