package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

@Composable
fun QuestTopBar(
    message: String?,
    operationLabel: String?,
    queueCount: Int,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(
                Brush.verticalGradient(
                    listOf(
                        VeteranQuestColors.bg2.copy(alpha = 0.95f),
                        VeteranQuestColors.bg1.copy(alpha = 0.95f),
                    ),
                ),
            )
            .border(1.dp, VeteranQuestColors.borderSoft, RoundedCornerShape(16.dp))
            .padding(horizontal = 14.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(11.dp))
                    .background(
                        Brush.linearGradient(
                            listOf(
                                VeteranQuestColors.accentStrong,
                                VeteranQuestColors.success,
                            ),
                        ),
                    )
                    .border(1.dp, VeteranQuestColors.text1.copy(alpha = 0.35f), RoundedCornerShape(11.dp)),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "V",
                    style = MaterialTheme.typography.titleMedium,
                    color = VeteranQuestColors.bg0,
                    fontWeight = FontWeight.Bold,
                )
            }

            Column {
                Text(
                    text = "Veteran Quest",
                    style = MaterialTheme.typography.headlineSmall,
                    color = VeteranQuestColors.text0,
                    fontWeight = FontWeight.SemiBold,
                )
                Text(
                    text = message ?: "Ready",
                    style = MaterialTheme.typography.bodyMedium,
                    color = VeteranQuestColors.text1,
                )
            }
        }

        Column(horizontalAlignment = Alignment.End) {
            Text(
                text = "Queue $queueCount",
                style = MaterialTheme.typography.labelLarge.copy(fontFamily = FontFamily.Monospace),
                color = VeteranQuestColors.accent,
            )
            Text(
                text = operationLabel ?: "No active operation",
                style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
                color = VeteranQuestColors.text2,
            )
        }
    }
}
