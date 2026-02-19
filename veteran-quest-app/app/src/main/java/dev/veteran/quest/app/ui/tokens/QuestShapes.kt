package dev.veteran.quest.app.ui.tokens

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Immutable
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp

@Immutable
data class QuestShapeTokens(
    val panel: RoundedCornerShape,
    val card: RoundedCornerShape,
    val button: RoundedCornerShape,
    val pill: RoundedCornerShape,
    val thumb: RoundedCornerShape,
    val input: RoundedCornerShape,
    val modCutout: Shape,
)

private class ModCutoutShape : Shape {
    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline {
        val path = Path().apply {
            moveTo(0f, size.height)
            lineTo(size.width, size.height)
            lineTo(size.width, 0f)
            close()
        }
        return Outline.Generic(path)
    }
}

val VeteranQuestShapes = QuestShapeTokens(
    panel = RoundedCornerShape(18.dp),
    card = RoundedCornerShape(14.dp),
    button = RoundedCornerShape(10.dp),
    pill = RoundedCornerShape(999.dp),
    thumb = RoundedCornerShape(10.dp),
    input = RoundedCornerShape(10.dp),
    modCutout = ModCutoutShape(),
)
