package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

@Composable
fun DiagnosticsPanel(
    logs: List<OperationLogEntry>,
    showDiagnostics: Boolean,
    onShowDiagnosticsChanged: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    var logLevel by remember { mutableStateOf("ALL") }
    var opFilter by remember { mutableStateOf("") }

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(
                text = "Diagnostics",
                style = MaterialTheme.typography.titleSmall,
                color = VeteranQuestColors.text0,
            )
            TextButton(onClick = { onShowDiagnosticsChanged(!showDiagnostics) }) {
                Text(if (showDiagnostics) "Hide" else "Show")
            }
        }

        if (!showDiagnostics) {
            Text(
                text = "Collapsed by default for comfort mode",
                style = MaterialTheme.typography.bodySmall,
                color = VeteranQuestColors.text2,
            )
            return@Column
        }

        val filteredLogs = logs.filter { entry ->
            (logLevel == "ALL" || entry.level == logLevel) &&
                (opFilter.isBlank() || entry.operationId.contains(opFilter, ignoreCase = true))
        }

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(VeteranQuestColors.bg1)
                .padding(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            OutlinedTextField(
                value = opFilter,
                onValueChange = { opFilter = it },
                singleLine = true,
                label = { Text("Operation id") },
                modifier = Modifier.fillMaxWidth(),
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                listOf("ALL", "INFO", "WARN", "ERROR").forEach { level ->
                    FilterChip(
                        selected = level == logLevel,
                        onClick = { logLevel = level },
                        label = { Text(level) },
                    )
                }
            }
        }

        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .height(240.dp)
                .border(1.dp, VeteranQuestColors.border, RoundedCornerShape(10.dp))
                .background(VeteranQuestColors.bg1, RoundedCornerShape(10.dp)),
            contentPadding = PaddingValues(bottom = 6.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            items(filteredLogs.take(250), key = { "${it.timestampMs}-${it.message}" }) { entry ->
                val tone = when (entry.level) {
                    "ERROR" -> VeteranQuestColors.danger
                    "WARN" -> VeteranQuestColors.warning
                    else -> VeteranQuestColors.text1
                }

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = entry.level,
                        color = tone,
                        style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
                        modifier = Modifier.weight(0.18f),
                    )
                    Text(
                        text = entry.stage,
                        color = VeteranQuestColors.text2,
                        style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
                        modifier = Modifier.weight(0.22f),
                    )
                    Text(
                        text = entry.message,
                        color = VeteranQuestColors.text1,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(0.60f),
                    )
                }
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(1.dp)
                        .background(VeteranQuestColors.border.copy(alpha = 0.35f)),
                )
            }
        }
    }
}
