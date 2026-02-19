package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.model.OperationLogEntry
import dev.veteran.quest.app.ui.tokens.MotionProfile
import dev.veteran.quest.app.ui.tokens.UiDensity
import dev.veteran.quest.app.ui.tokens.VeteranQuestColors
import dev.veteran.quest.model.LibraryFilter
import dev.veteran.quest.model.SortBy

@Composable
fun ControlRail(
    modifier: Modifier,
    search: String,
    sortBy: SortBy,
    sortAscending: Boolean,
    filter: LibraryFilter,
    syncing: Boolean,
    keepObb: Boolean,
    keepData: Boolean,
    keepAwake: Boolean,
    logs: List<OperationLogEntry>,
    showDiagnostics: Boolean,
    uiDensity: UiDensity,
    motionProfile: MotionProfile,
    onSearch: (String) -> Unit,
    onSort: (SortBy) -> Unit,
    onToggleSort: () -> Unit,
    onFilter: (LibraryFilter) -> Unit,
    onSync: () -> Unit,
    onKeepObb: (Boolean) -> Unit,
    onKeepData: (Boolean) -> Unit,
    onKeepAwake: (Boolean) -> Unit,
    onShowDiagnosticsChanged: (Boolean) -> Unit,
    onUiDensityChanged: (UiDensity) -> Unit,
    onMotionProfileChanged: (MotionProfile) -> Unit,
) {
    VeteranPanel(modifier = modifier.fillMaxHeight()) {
        LazyColumn(
            modifier = Modifier.fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item {
                OutlinedTextField(
                    value = search,
                    onValueChange = onSearch,
                    label = { Text("Search library") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            }

            item {
                Text("Sort", style = MaterialTheme.typography.titleSmall, color = VeteranQuestColors.text0)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    listOf(SortBy.POPULARITY, SortBy.NAME, SortBy.DATE, SortBy.SIZE).forEach { value ->
                        FilterChip(
                            selected = value == sortBy,
                            onClick = { onSort(value) },
                            label = { Text(value.name.lowercase().replaceFirstChar { it.uppercase() }) },
                        )
                    }
                }
            }

            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Button(onClick = onSync) {
                        Text(if (syncing) "Syncing..." else "Sync Catalog")
                    }
                    OutlinedButton(onClick = onToggleSort) {
                        Text(if (sortAscending) "Ascending" else "Descending")
                    }
                }
            }

            item {
                Text("Filter", style = MaterialTheme.typography.titleSmall, color = VeteranQuestColors.text0)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    listOf(LibraryFilter.NON_MODS, LibraryFilter.ALL, LibraryFilter.NEW, LibraryFilter.POPULAR).forEach { value ->
                        FilterChip(
                            selected = value == filter,
                            onClick = { onFilter(value) },
                            label = {
                                Text(
                                    value.name.lowercase()
                                        .replace('_', ' ')
                                        .replaceFirstChar { it.uppercase() },
                                )
                            },
                        )
                    }
                }
            }

            item {
                Text("Layout", style = MaterialTheme.typography.titleSmall, color = VeteranQuestColors.text0)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    FilterChip(
                        selected = uiDensity == UiDensity.COMFORTABLE,
                        onClick = { onUiDensityChanged(UiDensity.COMFORTABLE) },
                        label = { Text("Comfort") },
                    )
                    FilterChip(
                        selected = uiDensity == UiDensity.BALANCED,
                        onClick = { onUiDensityChanged(UiDensity.BALANCED) },
                        label = { Text("Balanced") },
                    )
                    FilterChip(
                        selected = motionProfile == MotionProfile.SUBTLE,
                        onClick = { onMotionProfileChanged(MotionProfile.SUBTLE) },
                        label = { Text("Subtle Motion") },
                    )
                    FilterChip(
                        selected = motionProfile == MotionProfile.MINIMAL,
                        onClick = { onMotionProfileChanged(MotionProfile.MINIMAL) },
                        label = { Text("Minimal") },
                    )
                }
            }

            items(
                listOf(
                    Triple("Keep OBB", keepObb, onKeepObb),
                    Triple("Keep Data", keepData, onKeepData),
                    Triple("Keep Awake", keepAwake, onKeepAwake),
                ),
                key = { it.first },
            ) { (label, checked, onChange) ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(label, color = VeteranQuestColors.text1, style = MaterialTheme.typography.bodyMedium)
                    Switch(checked = checked, onCheckedChange = onChange)
                }
            }

            item {
                DiagnosticsPanel(
                    logs = logs,
                    showDiagnostics = showDiagnostics,
                    onShowDiagnosticsChanged = onShowDiagnosticsChanged,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}
