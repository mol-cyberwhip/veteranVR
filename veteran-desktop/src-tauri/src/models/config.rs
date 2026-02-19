use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
pub struct PublicConfig {
    pub base_uri: String,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("password must be a base64 string")]
    InvalidFormat,
    #[error("password field is not valid base64-encoded UTF-8")]
    Base64Error(#[from] base64::DecodeError),
    #[error("utf8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl PublicConfig {
    pub fn from_json(data: &Value) -> Result<Self, ConfigError> {
        let raw_password = data.get("password").and_then(|v| v.as_str()).unwrap_or("");

        let decoded_bytes = general_purpose::STANDARD.decode(raw_password)?;
        let decoded_password = String::from_utf8(decoded_bytes)?;

        let base_uri = data
            .get("baseUri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(PublicConfig {
            base_uri,
            password: decoded_password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_config() {
        let password = "secret_password";
        let encoded = general_purpose::STANDARD.encode(password);

        let json = json!({
            "baseUri": "http://example.com",
            "password": encoded
        });

        let config = PublicConfig::from_json(&json).unwrap();
        assert_eq!(config.password, password);
        assert_eq!(config.base_uri, "http://example.com");
    }

    #[test]
    fn test_invalid_base64() {
        let json = json!({
            "baseUri": "http://example.com",
            "password": "!!!not_base64!!!"
        });

        assert!(PublicConfig::from_json(&json).is_err());
    }

    #[test]
    fn test_missing_fields_defaults() {
        // Base64 of empty string is empty string
        let json = json!({});
        // Depending on implementation, missing password might fail decode if not handled.
        // My implementation reads empty string if missing, which decodes to empty string.
        let config = PublicConfig::from_json(&json).unwrap();
        assert_eq!(config.password, "");
        assert_eq!(config.base_uri, "");
    }
}
