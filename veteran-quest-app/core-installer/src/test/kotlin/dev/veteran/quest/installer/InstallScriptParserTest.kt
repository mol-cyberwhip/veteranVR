package dev.veteran.quest.installer

import com.google.common.truth.Truth.assertThat
import org.junit.Test

class InstallScriptParserTest {
    @Test
    fun `parse keeps supported commands and warns on unsupported`() {
        val plan = InstallScriptParser.parse(
            """
            adb install base.apk
            adb push com.pkg /sdcard/Android/obb/
            adb shell pm grant x y
            adb wait-for-device
            notadb hello
            """.trimIndent()
        )

        assertThat(plan.actions).hasSize(3)
        assertThat(plan.warnings).hasSize(2)
    }
}
