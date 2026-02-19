package dev.veteran.quest.installer

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.nio.file.Files

class InstallPlannerTest {
    @Test
    fun `plan uses install txt when present`() {
        val dir = Files.createTempDirectory("install-plan-test").toFile()
        dir.resolve("install.txt").writeText("adb shell echo hi")
        dir.resolve("base.apk").writeText("x")

        val plan = InstallPlanner.planFromExtractedGameDirectory(dir, "pkg")

        assertThat(plan.actions).hasSize(1)
        assertThat(plan.actions.first()).isInstanceOf(InstallAction.Shell::class.java)
    }

    @Test
    fun `plan falls back to apk and obb`() {
        val dir = Files.createTempDirectory("install-plan-fallback").toFile()
        dir.resolve("my.apk").writeText("x")
        dir.resolve("com.test.pkg").mkdirs()

        val plan = InstallPlanner.planFromExtractedGameDirectory(dir, "com.test.pkg")

        assertThat(plan.actions).hasSize(2)
        assertThat(plan.actions[0]).isInstanceOf(InstallAction.InstallApk::class.java)
        assertThat(plan.actions[1]).isInstanceOf(InstallAction.PushDirectory::class.java)
    }
}
