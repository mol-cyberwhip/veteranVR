package dev.veteran.quest.app.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

@Composable
fun OperationStrip(
    operation: DownloadOperation?,
    onPause: (DownloadOperation) -> Unit,
    onResume: (DownloadOperation) -> Unit,
) {
    AnimatedVisibility(
        visible = operation != null,
        enter = fadeIn(),
        exit = fadeOut(),
    ) {
        if (operation == null) return@AnimatedVisibility

        val pillColor = when (operation.state) {
            DownloadState.FAILED -> VeteranQuestColors.danger
            DownloadState.PAUSED -> VeteranQuestColors.warning
            DownloadState.COMPLETED -> VeteranQuestColors.success
            else -> VeteranQuestColors.accent
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(VeteranQuestColors.bg1.copy(alpha = 0.88f))
                .border(1.dp, VeteranQuestColors.borderSoft, RoundedCornerShape(12.dp))
                .padding(horizontal = 12.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = operation.releaseName,
                style = MaterialTheme.typography.bodyMedium,
                color = VeteranQuestColors.text0,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )

            Text(
                text = "${operation.state.name.lowercase()} ${operation.progressPercent.toInt()}%",
                style = MaterialTheme.typography.labelLarge.copy(fontFamily = FontFamily.Monospace),
                color = pillColor,
            )

            Box(
                modifier = Modifier
                    .width(120.dp)
                    .height(4.dp)
                    .clip(RoundedCornerShape(999.dp))
                    .background(VeteranQuestColors.bg2),
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth((operation.progressPercent / 100.0).coerceIn(0.0, 1.0).toFloat())
                        .height(4.dp)
                        .background(pillColor),
                )
            }

            if (operation.state == DownloadState.DOWNLOADING) {
                OutlinedButton(onClick = { onPause(operation) }) {
                    Text("Pause")
                }
            }
            if (operation.state == DownloadState.PAUSED) {
                Button(onClick = { onResume(operation) }) {
                    Text("Resume")
                }
            }
        }
    }
}
