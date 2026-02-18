package dev.veteran.quest.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.veteran.quest.model.Game
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.SortBy

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuestAppScreen(viewModel: QuestViewModel) {
    val state by viewModel.state.collectAsState()

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                brush = Brush.linearGradient(
                    0.0f to Color(0xFF0B1120),
                    0.5f to Color(0xFF12243F),
                    1.0f to Color(0xFF0C182D),
                )
            )
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp)
        ) {
            Header(state.message)
            Spacer(modifier = Modifier.height(12.dp))

            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                OutlinedTextField(
                    value = state.search,
                    onValueChange = viewModel::onSearchChanged,
                    modifier = Modifier.weight(1f),
                    label = { Text("Search catalog") },
                    singleLine = true,
                )
                Button(onClick = { viewModel.refreshCatalog(force = true) }) {
                    Text("Sync")
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(
                    selected = state.sortBy == SortBy.POPULARITY,
                    onClick = { viewModel.onSortByChanged(SortBy.POPULARITY) },
                    label = { Text("Popularity") },
                )
                FilterChip(
                    selected = state.sortBy == SortBy.NAME,
                    onClick = { viewModel.onSortByChanged(SortBy.NAME) },
                    label = { Text("Name") },
                )
                FilterChip(
                    selected = state.sortBy == SortBy.DATE,
                    onClick = { viewModel.onSortByChanged(SortBy.DATE) },
                    label = { Text("Date") },
                )
                FilterChip(
                    selected = state.sortBy == SortBy.SIZE,
                    onClick = { viewModel.onSortByChanged(SortBy.SIZE) },
                    label = { Text("Size") },
                )
                TextButton(onClick = viewModel::onSortDirectionToggled) {
                    Text(if (state.sortAscending) "Asc" else "Desc")
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(
                    selected = state.filter == LibraryFilter.NON_MODS,
                    onClick = { viewModel.onFilterChanged(LibraryFilter.NON_MODS) },
                    label = { Text("Non-Mods") },
                )
                FilterChip(
                    selected = state.filter == LibraryFilter.ALL,
                    onClick = { viewModel.onFilterChanged(LibraryFilter.ALL) },
                    label = { Text("All") },
                )
                FilterChip(
                    selected = state.filter == LibraryFilter.NEW,
                    onClick = { viewModel.onFilterChanged(LibraryFilter.NEW) },
                    label = { Text("New") },
                )
                FilterChip(
                    selected = state.filter == LibraryFilter.POPULAR,
                    onClick = { viewModel.onFilterChanged(LibraryFilter.POPULAR) },
                    label = { Text("Popular") },
                )
            }

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Checkbox(checked = state.keepObbOnUninstall, onCheckedChange = viewModel::onKeepObbChanged)
                Text("Keep OBB")
                Checkbox(checked = state.keepDataOnUninstall, onCheckedChange = viewModel::onKeepDataChanged)
                Text("Keep Data")
            }

            Spacer(modifier = Modifier.height(10.dp))

            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
                contentPadding = PaddingValues(bottom = 32.dp),
            ) {
                items(state.games, key = { "${it.packageName}-${it.releaseName}" }) { game ->
                    GameCard(
                        game = game,
                        onInstall = { viewModel.install(game) },
                        onUninstall = { viewModel.uninstall(game) },
                    )
                }
            }
        }
    }
}

@Composable
private fun Header(message: String?) {
    Column {
        Text(
            text = "Veteran Quest",
            style = MaterialTheme.typography.headlineMedium.copy(fontWeight = FontWeight.Bold),
            color = Color(0xFFE2F4FF),
        )
        Text(
            text = "Catalog, browse, install/update from headset",
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFF9CC4E7),
        )
        if (!message.isNullOrBlank()) {
            Text(
                text = message,
                style = MaterialTheme.typography.bodySmall,
                color = Color(0xFF87F6C9),
            )
        }
    }
}

@Composable
private fun GameCard(game: Game, onInstall: () -> Unit, onUninstall: () -> Unit) {
    Card(
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = Color(0x221A2F4A)),
        modifier = Modifier.fillMaxWidth(),
    ) {
        Column(modifier = Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(
                text = game.gameName,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                color = Color(0xFFF5FAFF),
            )
            Text(
                text = game.releaseName,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                color = Color(0xFFB3D2ED),
            )
            Text(
                text = "${game.size}  •  ${game.downloads} dl",
                style = MaterialTheme.typography.labelMedium,
                color = Color(0xFF7AC8FF),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onInstall) { Text("Install") }
                TextButton(onClick = onUninstall) { Text("Uninstall") }
            }
        }
    }
}
