package dev.veteran.quest.app.service

import dev.veteran.quest.app.model.CatalogSnapshot

interface CatalogSyncService {
    suspend fun syncCatalog(force: Boolean): Result<CatalogSnapshot>
}
