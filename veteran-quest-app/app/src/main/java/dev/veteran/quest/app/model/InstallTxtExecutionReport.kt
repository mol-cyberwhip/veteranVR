package dev.veteran.quest.app.model

data class InstallTxtExecutionReport(
    val warnings: List<String>,
    val executedActions: Int,
)
