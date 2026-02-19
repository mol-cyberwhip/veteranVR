package dev.veteran.quest.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.veteran.quest.app.ui.QuestAppScreen
import dev.veteran.quest.app.ui.QuestViewModel
import dev.veteran.quest.app.ui.theme.VeteranQuestTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val container = AppContainer.from(this)

        setContent {
            VeteranQuestTheme {
                val vm: QuestViewModel = viewModel(
                    factory = QuestViewModel.factory(container),
                )
                QuestAppScreen(
                    viewModel = vm,
                    permissionGateService = container.permissionGateService,
                )
            }
        }
    }
}
