package dev.veteran.quest.app.ui.tokens

import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

enum class UiDensity {
    COMFORTABLE,
    BALANCED,
}

enum class MotionProfile {
    SUBTLE,
    MINIMAL,
}

data class QuestColorTokens(
    val bg0: Color,
    val bg1: Color,
    val bg2: Color,
    val panelTop: Color,
    val panelBottom: Color,
    val border: Color,
    val borderSoft: Color,
    val text0: Color,
    val text1: Color,
    val text2: Color,
    val accent: Color,
    val accentStrong: Color,
    val accentMuted: Color,
    val success: Color,
    val successMuted: Color,
    val warning: Color,
    val warningMuted: Color,
    val danger: Color,
    val dangerMuted: Color,
    val modCutout: Color,
) {
    val appBackgroundBrush: Brush
        get() = Brush.linearGradient(
            listOf(
                bg0,
                Color(0xFF0D1320),
                Color(0xFF111A2C),
            ),
        )

    val panelBrush: Brush
        get() = Brush.linearGradient(listOf(panelTop, panelBottom))
}

data class QuestSpacingTokens(
    val outerPadding: Dp,
    val sectionGap: Dp,
    val panelPadding: Dp,
    val railWidth: Dp,
    val cardGap: Dp,
    val cardThumbHeight: Dp,
    val buttonHeight: Dp,
)

data class QuestElevationTokens(
    val panelShadowAlpha: Float,
    val cardShadowAlpha: Float,
)

val VeteranQuestColors = QuestColorTokens(
    bg0 = Color(0xFF090D15),
    bg1 = Color(0xFF0D1320),
    bg2 = Color(0xFF111A2C),
    panelTop = Color(0xE01D2638),
    panelBottom = Color(0xE0141D2C),
    border = Color(0xA0334666),
    borderSoft = Color(0x80335276),
    text0 = Color(0xFFF0F6FF),
    text1 = Color(0xFFBED0EE),
    text2 = Color(0xFF7F94B8),
    accent = Color(0xFF5AA3FF),
    accentStrong = Color(0xFF2A7BE4),
    accentMuted = Color(0x264A90D9),
    success = Color(0xFF34D399),
    successMuted = Color(0x2634D399),
    warning = Color(0xFFF4B548),
    warningMuted = Color(0x26F4B548),
    danger = Color(0xFFF1737F),
    dangerMuted = Color(0x26F1737F),
    modCutout = Color(0xFFCC5A1E),
)

fun spacingForDensity(density: UiDensity): QuestSpacingTokens {
    return when (density) {
        UiDensity.COMFORTABLE -> QuestSpacingTokens(
            outerPadding = 14.dp,
            sectionGap = 10.dp,
            panelPadding = 12.dp,
            railWidth = 376.dp,
            cardGap = 10.dp,
            cardThumbHeight = 124.dp,
            buttonHeight = 44.dp,
        )

        UiDensity.BALANCED -> QuestSpacingTokens(
            outerPadding = 12.dp,
            sectionGap = 8.dp,
            panelPadding = 10.dp,
            railWidth = 344.dp,
            cardGap = 8.dp,
            cardThumbHeight = 112.dp,
            buttonHeight = 40.dp,
        )
    }
}

val VeteranQuestElevations = QuestElevationTokens(
    panelShadowAlpha = 0.40f,
    cardShadowAlpha = 0.45f,
)
