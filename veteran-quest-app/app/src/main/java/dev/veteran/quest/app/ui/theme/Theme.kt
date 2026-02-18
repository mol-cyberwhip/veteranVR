package dev.veteran.quest.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val Colors = darkColorScheme(
    primary = Color(0xFF33D5A5),
    onPrimary = Color(0xFF02140F),
    secondary = Color(0xFF77B8FF),
    background = Color(0xFF0A1323),
    surface = Color(0xFF101C31),
)

@Composable
fun VeteranQuestTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = Colors,
        content = content,
    )
}
