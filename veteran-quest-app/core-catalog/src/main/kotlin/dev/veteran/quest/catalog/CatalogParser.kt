package dev.veteran.quest.catalog

import dev.veteran.quest.model.Game
import java.time.Duration
import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneOffset

object CatalogParser {
    fun parseGameList(content: String, now: Instant = Instant.now()): CatalogDataset {
        val gamesByKey = linkedMapOf<Pair<String, String>, Game>()
        val allVersions = mutableListOf<Game>()
        val popularityScores = mutableMapOf<String, Double>()

        content.lineSequence()
            .drop(1)
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .forEach { line ->
                val fields = line.split(';')
                val parsed = parseRow(fields, now) ?: return@forEach
                allVersions += parsed

                parsed.downloads.toDoubleOrNull()?.let { score ->
                    val existing = popularityScores[parsed.packageName]
                    if (existing == null || score > existing) {
                        popularityScores[parsed.packageName] = score
                    }
                }

                val key = parsed.packageName to parsed.gameName
                val existing = gamesByKey[key]
                if (existing == null || isHigherVersion(parsed, existing)) {
                    gamesByKey[key] = parsed
                }
            }

        val rankings = popularityScores
            .filterValues { it > 0.0 }
            .toList()
            .sortedByDescending { (_, score) -> score }
            .mapIndexed { idx, (pkg, _) -> pkg to (idx + 1) }
            .toMap()

        val latestWithPopularity = gamesByKey.values.map { game ->
            game.copy(popularityRank = rankings[game.packageName] ?: 0)
        }

        return CatalogDataset(
            latestGames = latestWithPopularity,
            allVersions = allVersions,
        )
    }

    private fun parseRow(fields: List<String>, now: Instant): Game? {
        if (fields.size < 4) {
            return null
        }

        val gameName = fields.getOrNull(0).orEmpty().trim()
        val releaseName = fields.getOrNull(1).orEmpty().trim()
        val packageName = fields.getOrNull(2).orEmpty().trim()
        val versionCode = fields.getOrNull(3).orEmpty().trim()

        if (looksLikeModernSchema(fields)) {
            val lastUpdated = fields.getOrNull(4).orEmpty().trim()
            val size = normalizeSizeField(fields.getOrNull(5).orEmpty().trim())
            val downloads = fields.getOrNull(6).orEmpty().trim()
            val versionName = extractVersionNameFromRelease(releaseName)

            return Game(
                gameName = gameName,
                releaseName = releaseName,
                packageName = packageName,
                versionCode = versionCode,
                versionName = versionName,
                downloads = downloads,
                size = size,
                lastUpdated = lastUpdated,
                isNew = isNewGame(lastUpdated, now),
            )
        }

        return Game(
            gameName = gameName,
            releaseName = releaseName,
            packageName = packageName,
            versionCode = versionCode,
            releaseApkPath = fields.getOrNull(4).orEmpty().trim(),
            versionName = fields.getOrNull(5).orEmpty().trim(),
            downloads = fields.getOrNull(6).orEmpty().trim(),
            size = fields.getOrNull(7).orEmpty().trim(),
            lastUpdated = fields.getOrNull(8).orEmpty().trim(),
            isNew = isNewGame(fields.getOrNull(8).orEmpty().trim(), now),
        )
    }

    private fun isHigherVersion(candidate: Game, existing: Game): Boolean {
        val existingVer = existing.versionCode.toLongOrNull() ?: 0L
        val candidateVer = candidate.versionCode.toLongOrNull() ?: 0L

        return when {
            candidateVer > existingVer -> true
            candidateVer < existingVer -> false
            else -> candidate.versionCode > existing.versionCode
        }
    }

    private fun looksLikeModernSchema(fields: List<String>): Boolean {
        if (fields.size < 7) {
            return false
        }

        val datePrefix = fields[4].trim()
        if (datePrefix.length < 10 || datePrefix[4] != '-' || datePrefix[7] != '-') {
            return false
        }

        val sizeField = fields[5].trim().uppercase()
        return sizeField.toDoubleOrNull() != null || sizeField.matches(Regex("^\\d+(\\.\\d+)?\\s*(KB|MB|GB|TB|KIB|MIB|GIB|TIB)$"))
    }

    private fun extractVersionNameFromRelease(releaseName: String): String {
        val match = Regex("(?i)\\bv\\d+\\+([^\\s-]+)").find(releaseName) ?: return ""
        return match.groupValues.getOrNull(1).orEmpty().trim()
    }

    private fun normalizeSizeField(rawSize: String): String {
        val sizeValue = rawSize.trim()
        if (sizeValue.isEmpty()) {
            return ""
        }

        val unitPattern = Regex("(?i)^\\d+(?:\\.\\d+)?\\s*(KB|MB|GB|TB|KIB|MIB|GIB|TIB)$")
        if (unitPattern.matches(sizeValue)) {
            return sizeValue
        }

        val asNumber = sizeValue.toDoubleOrNull() ?: return sizeValue
        val normalized = if (asNumber % 1.0 == 0.0) asNumber.toLong().toString() else asNumber.toString()
        return "$normalized MB"
    }

    private fun isNewGame(lastUpdated: String, now: Instant): Boolean {
        val timestamp = Regex("^(\\d{4}-\\d{2}-\\d{2})").find(lastUpdated)?.groupValues?.get(1) ?: return false
        return runCatching {
            val dt = LocalDateTime.parse("${timestamp}T00:00:00")
            Duration.between(dt.toInstant(ZoneOffset.UTC), now).toDays() <= 30
        }.getOrDefault(false)
    }
}
