package dev.veteran.quest.model

import com.google.common.truth.Truth.assertThat
import org.junit.Test

class GameTest {
    @Test
    fun `isModded detects common markers`() {
        val game = Game(
            gameName = "Beat Saber",
            releaseName = "Beat Saber v1.2 -MOD",
            packageName = "com.beatgames.beatsaber",
            versionCode = "12",
        )

        assertThat(game.isModded).isTrue()
    }
}
