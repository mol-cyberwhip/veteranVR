package dev.veteran.quest.installer

import java.io.File

object InstallPlanner {
    fun planFromExtractedGameDirectory(gameDir: File, packageName: String): InstallPlan {
        val installTxt = listOf("install.txt", "Install.txt")
            .asSequence()
            .map { File(gameDir, it) }
            .firstOrNull { it.exists() }

        if (installTxt != null) {
            return InstallScriptParser.parse(installTxt.readText())
        }

        val firstApk = gameDir.listFiles()
            ?.filter { it.isFile && it.extension.equals("apk", ignoreCase = true) }
            ?.sortedBy { it.name }
            ?.firstOrNull()
            ?: return InstallPlan(emptyList(), listOf("No APK found in ${gameDir.absolutePath}"))

        val actions = mutableListOf<InstallAction>(
            InstallAction.InstallApk(firstApk.name),
        )

        val obbDir = File(gameDir, packageName)
        if (obbDir.isDirectory) {
            actions += InstallAction.PushDirectory(
                relativeLocalPath = obbDir.name,
                remotePath = "/sdcard/Android/obb/",
            )
        }

        return InstallPlan(actions = actions)
    }
}
