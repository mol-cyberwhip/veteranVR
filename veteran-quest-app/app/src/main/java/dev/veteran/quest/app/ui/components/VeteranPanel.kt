package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors
import dev.veteran.quest.app.ui.tokens.VeteranQuestElevations
import dev.veteran.quest.app.ui.tokens.VeteranQuestShapes

@Composable
fun VeteranPanel(
    modifier: Modifier = Modifier,
    padding: Dp = 12.dp,
    brush: Brush = VeteranQuestColors.panelBrush,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier = modifier
            .shadow(
                elevation = 12.dp,
                shape = VeteranQuestShapes.panel,
                ambientColor = VeteranQuestColors.bg0.copy(alpha = VeteranQuestElevations.panelShadowAlpha),
                spotColor = VeteranQuestColors.bg0.copy(alpha = VeteranQuestElevations.panelShadowAlpha),
            )
            .clip(VeteranQuestShapes.panel)
            .background(brush)
            .border(1.dp, VeteranQuestColors.border, VeteranQuestShapes.panel)
            .padding(padding),
        content = content,
    )
}
