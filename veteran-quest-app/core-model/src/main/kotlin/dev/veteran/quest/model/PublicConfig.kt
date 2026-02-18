package dev.veteran.quest.model

import kotlinx.serialization.Serializable

@Serializable
data class PublicConfig(
    val baseUri: String,
    val password: String,
)
