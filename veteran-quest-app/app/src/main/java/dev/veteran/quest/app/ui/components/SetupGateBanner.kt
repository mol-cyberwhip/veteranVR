package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

@Composable
fun SetupGateBanner(
    hasInstall: Boolean,
    hasFiles: Boolean,
    freeBytes: Long,
    minBytes: Long,
    onOpenInstallSettings: () -> Unit,
    onOpenFilesSettings: () -> Unit,
    onRefresh: () -> Unit,
    humanBytes: (Long) -> String,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(VeteranQuestColors.warningMuted.copy(alpha = 0.55f))
            .border(1.dp, VeteranQuestColors.warning.copy(alpha = 0.5f), RoundedCornerShape(14.dp))
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(
            text = "Setup Required For Installs",
            style = MaterialTheme.typography.titleMedium,
            color = VeteranQuestColors.warning,
        )

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Install perm: ${if (hasInstall) "OK" else "Missing"}",
                color = if (hasInstall) VeteranQuestColors.success else VeteranQuestColors.warning,
                style = MaterialTheme.typography.bodyMedium,
            )
            Text(
                text = "Files perm: ${if (hasFiles) "OK" else "Missing"}",
                color = if (hasFiles) VeteranQuestColors.success else VeteranQuestColors.warning,
                style = MaterialTheme.typography.bodyMedium,
            )
            Text(
                text = "Free ${humanBytes(freeBytes)} / Need ${humanBytes(minBytes)}",
                color = VeteranQuestColors.text1,
                style = MaterialTheme.typography.labelLarge.copy(fontFamily = FontFamily.Monospace),
            )
        }

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(onClick = onOpenInstallSettings) { Text("Install Settings") }
            OutlinedButton(onClick = onOpenFilesSettings) { Text("Storage Settings") }
            Button(onClick = onRefresh) { Text("Recheck") }
        }
    }
}
