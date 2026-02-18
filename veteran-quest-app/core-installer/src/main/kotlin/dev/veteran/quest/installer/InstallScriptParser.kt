package dev.veteran.quest.installer

object InstallScriptParser {
    fun parse(scriptContent: String): InstallPlan {
        val actions = mutableListOf<InstallAction>()
        val warnings = mutableListOf<String>()

        scriptContent.lineSequence().forEachIndexed { index, rawLine ->
            val line = rawLine.trim()
            if (line.isEmpty()) {
                return@forEachIndexed
            }
            if (!line.startsWith("adb")) {
                warnings += "Line ${index + 1}: ignored non-adb command '$line'"
                return@forEachIndexed
            }

            val args = line.removePrefix("adb").trim().split(Regex("\\s+")).filter { it.isNotBlank() }
            if (args.isEmpty()) {
                warnings += "Line ${index + 1}: empty adb command"
                return@forEachIndexed
            }

            when (args.first()) {
                "install" -> {
                    val apk = args.lastOrNull()
                    if (apk == null) {
                        warnings += "Line ${index + 1}: install missing apk path"
                    } else {
                        actions += InstallAction.InstallApk(apk)
                    }
                }

                "push" -> {
                    if (args.size < 3) {
                        warnings += "Line ${index + 1}: push missing arguments"
                    } else {
                        actions += InstallAction.PushDirectory(
                            relativeLocalPath = args[1],
                            remotePath = args[2],
                        )
                    }
                }

                "shell" -> {
                    val command = args.drop(1).joinToString(" ")
                    if (command.isBlank()) {
                        warnings += "Line ${index + 1}: shell missing command"
                    } else {
                        actions += InstallAction.Shell(command)
                    }
                }

                else -> warnings += "Line ${index + 1}: unsupported adb command '${args.first()}', continuing"
            }
        }

        return InstallPlan(actions = actions, warnings = warnings)
    }
}
