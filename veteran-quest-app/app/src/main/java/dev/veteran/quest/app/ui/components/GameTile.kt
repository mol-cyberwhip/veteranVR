package dev.veteran.quest.app.ui.components

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.DownloadState
import dev.veteran.quest.app.model.LibraryItemUi
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors
import dev.veteran.quest.app.ui.tokens.VeteranQuestShapes
import java.io.File

@Composable
fun GameTile(
    item: LibraryItemUi,
    operation: DownloadOperation?,
    thumbHeight: Dp,
    onInstall: () -> Unit,
    onUninstall: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, VeteranQuestColors.border, VeteranQuestShapes.card)
            .clip(VeteranQuestShapes.card)
            .background(
                Brush.verticalGradient(
                    listOf(
                        VeteranQuestColors.bg1.copy(alpha = 0.95f),
                        VeteranQuestColors.bg2.copy(alpha = 0.95f),
                    ),
                ),
            )
            .padding(9.dp),
        verticalArrangement = Arrangement.spacedBy(7.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(thumbHeight)
                .clip(VeteranQuestShapes.thumb)
                .background(
                    Brush.linearGradient(
                        listOf(
                            VeteranQuestColors.accentStrong.copy(alpha = 0.6f),
                            VeteranQuestColors.bg2,
                        ),
                    ),
                ),
        ) {
            val thumbModel = if (item.thumbnailExists) File(item.thumbnailPath) else null
            if (thumbModel != null) {
                AsyncImage(
                    model = thumbModel,
                    contentDescription = item.game.gameName,
                    modifier = Modifier.fillMaxSize(),
                )
            } else {
                Text(
                    text = item.game.gameName.take(2).uppercase(),
                    color = VeteranQuestColors.text1,
                    modifier = Modifier.align(Alignment.Center),
                    style = MaterialTheme.typography.titleMedium,
                )
            }

            Box(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .height(36.dp)
                    .background(
                        Brush.verticalGradient(
                            listOf(Color.Transparent, Color.Black.copy(alpha = 0.60f)),
                        ),
                    ),
            )

            if (item.game.isModded) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .width(88.dp)
                        .height(28.dp)
                        .clip(VeteranQuestShapes.modCutout)
                        .background(VeteranQuestColors.modCutout),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "MOD",
                        style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
                        color = VeteranQuestColors.text0,
                    )
                }
            }

            if (operation?.state == DownloadState.DOWNLOADING) {
                val progress = (operation.progressPercent / 100.0).coerceIn(0.0, 1.0).toFloat()
                val shimmer by rememberInfiniteTransition(label = "progress-shimmer").animateFloat(
                    initialValue = -1f,
                    targetValue = 2f,
                    animationSpec = infiniteRepeatable(
                        animation = tween(1300, easing = LinearEasing),
                        repeatMode = RepeatMode.Restart,
                    ),
                    label = "progress-shimmer-offset",
                )

                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(4.dp)
                        .background(VeteranQuestColors.bg0.copy(alpha = 0.6f)),
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(progress)
                            .height(4.dp)
                            .background(VeteranQuestColors.accent),
                    )
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp)
                            .background(
                                Brush.horizontalGradient(
                                    colors = listOf(
                                        Color.Transparent,
                                        Color.White.copy(alpha = 0.35f),
                                        Color.Transparent,
                                    ),
                                    startX = shimmer * 220f,
                                    endX = shimmer * 220f + 90f,
                                ),
                            ),
                    )
                }
            }
        }

        Text(
            text = item.game.gameName,
            style = MaterialTheme.typography.titleSmall,
            color = VeteranQuestColors.text0,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )

        Text(
            text = item.game.releaseName,
            style = MaterialTheme.typography.bodySmall,
            color = VeteranQuestColors.text2,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )

        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            StatPill(text = item.game.size.ifBlank { "size ?" }, color = VeteranQuestColors.accent)
            if (item.isInstalled) {
                StatPill(text = "installed", color = VeteranQuestColors.success)
            }
            if (operation != null) {
                val tone = when (operation.state) {
                    DownloadState.FAILED -> VeteranQuestColors.danger
                    DownloadState.PAUSED -> VeteranQuestColors.warning
                    DownloadState.COMPLETED -> VeteranQuestColors.success
                    else -> VeteranQuestColors.accent
                }
                StatPill(text = operation.state.name.lowercase(), color = tone)
            }
        }

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onInstall, modifier = Modifier.weight(1f)) {
                Text("Install")
            }
            OutlinedButton(onClick = onUninstall, modifier = Modifier.weight(1f)) {
                Text("Uninstall")
            }
        }
    }
}

@Composable
private fun StatPill(text: String, color: Color) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(color.copy(alpha = 0.16f))
            .border(1.dp, color.copy(alpha = 0.85f), RoundedCornerShape(999.dp))
            .padding(horizontal = 8.dp, vertical = 2.dp),
    ) {
        Text(
            text = text,
            color = color,
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
        )
    }
}
