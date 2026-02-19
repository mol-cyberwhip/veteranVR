package dev.veteran.quest.catalog

import java.security.MessageDigest

object CatalogHash {
    fun gameNameToHash(releaseName: String): String {
        val digest = MessageDigest.getInstance("MD5")
        val input = "$releaseName\n".toByteArray(Charsets.UTF_8)
        val hash = digest.digest(input)
        return hash.joinToString(separator = "") { "%02x".format(it) }
    }
}
