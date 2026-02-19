package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

@Composable
fun TelemetryBar(
    activeOperation: DownloadOperation?,
    queueCount: Int,
    totalLogs: Int,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(VeteranQuestColors.bg1.copy(alpha = 0.96f))
            .border(1.dp, VeteranQuestColors.borderSoft, RoundedCornerShape(10.dp))
            .padding(horizontal = 12.dp, vertical = 7.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = activeOperation?.let { "Active: ${it.releaseName}" } ?: "Active: none",
            color = VeteranQuestColors.text1,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.weight(1f),
        )
        Text(
            text = "queue=$queueCount logs=$totalLogs",
            color = VeteranQuestColors.text2,
            style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
        )
    }
}
