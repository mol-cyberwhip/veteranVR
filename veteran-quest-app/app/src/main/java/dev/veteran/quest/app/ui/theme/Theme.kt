package dev.veteran.quest.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import dev.veteran.quest.app.R

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
    primary = Color(0xFF33D5A5),
    onPrimary = Color(0xFF02140F),
    secondary = Color(0xFF77B8FF),
    background = Color(0xFF0A1323),
    surface = Color(0xFF101C31),
)

private val VeteranTypography = Typography(
    headlineSmall = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 30.sp),
    titleMedium = TextStyle(fontFamily = rajdhani, fontWeight = FontWeight.SemiBold, fontSize = 20.sp),
    bodyMedium = TextStyle(fontFamily = atkinson, fontWeight = FontWeight.Normal, fontSize = 15.sp),
    bodySmall = TextStyle(fontFamily = atkinson, fontWeight = FontWeight.Normal, fontSize = 13.sp),
    labelLarge = TextStyle(fontFamily = jetBrainsMono, fontWeight = FontWeight.Normal, fontSize = 13.sp),
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
