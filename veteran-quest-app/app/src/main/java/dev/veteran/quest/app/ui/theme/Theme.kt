package dev.veteran.quest.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import dev.veteran.quest.app.R
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors

private val rajdhani = FontFamily(
    Font(R.font.rajdhani_regular, FontWeight.Normal),
    Font(R.font.rajdhani_semibold, FontWeight.SemiBold),
)

private val atkinson = FontFamily(
    Font(R.font.atkinson_hyperlegible_regular, FontWeight.Normal),
)

private val jetBrainsMono = FontFamily(
    Font(R.font.jetbrains_mono_wght, FontWeight.Normal),
)

private val Colors = darkColorScheme(
    primary = VeteranQuestColors.accent,
    onPrimary = VeteranQuestColors.bg0,
    secondary = VeteranQuestColors.accentStrong,
    background = VeteranQuestColors.bg0,
    surface = VeteranQuestColors.bg1,
    onSurface = VeteranQuestColors.text0,
)

private val VeteranTypography = Typography(
    headlineLarge = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 40.sp),
    headlineSmall = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 32.sp),
    titleLarge = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 24.sp),
    titleMedium = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 21.sp),
    titleSmall = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 19.sp),
    bodyLarge = TextStyle(fontFamily = atkinson, fontWeight = FontWeight.Normal, fontSize = 18.sp),
    bodyMedium = TextStyle(fontFamily = atkinson, fontWeight = FontWeight.Normal, fontSize = 16.sp),
    bodySmall = TextStyle(fontFamily = atkinson, fontWeight = FontWeight.Normal, fontSize = 14.sp),
    labelLarge = TextStyle(fontFamily = jetBrainsMono, fontWeight = FontWeight.Normal, fontSize = 14.sp),
    labelMedium = TextStyle(fontFamily = jetBrainsMono, fontWeight = FontWeight.Normal, fontSize = 12.sp),
    labelSmall = TextStyle(fontFamily = jetBrainsMono, fontWeight = FontWeight.Normal, fontSize = 11.sp),
)

@Composable
fun VeteranQuestTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = Colors,
        typography = VeteranTypography,
        content = content,
    )
}
