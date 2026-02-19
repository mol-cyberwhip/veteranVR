package dev.veteran.quest.app.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.veteran.quest.app.model.DownloadOperation
import dev.veteran.quest.app.model.LibraryItemUi

@Composable
fun CatalogGrid(
    items: List<LibraryItemUi>,
    operations: List<DownloadOperation>,
    cardGap: Dp,
    thumbHeight: Dp,
    onInstall: (LibraryItemUi) -> Unit,
    onUninstall: (LibraryItemUi) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 228.dp),
        modifier = modifier
            .fillMaxSize()
            .padding(2.dp),
        horizontalArrangement = Arrangement.spacedBy(cardGap),
        verticalArrangement = Arrangement.spacedBy(cardGap),
        contentPadding = PaddingValues(bottom = 14.dp),
    ) {
        items(items, key = { "${it.game.packageName}-${it.game.releaseName}" }) { item ->
            val op = operations.firstOrNull { it.packageName == item.game.packageName }
            GameTile(
                item = item,
                operation = op,
                thumbHeight = thumbHeight,
                onInstall = { onInstall(item) },
                onUninstall = { onUninstall(item) },
            )
        }
    }
}
