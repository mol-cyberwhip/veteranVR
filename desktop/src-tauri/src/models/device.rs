use serde::{Deserialize, Serialize};

/// Internal device representation from ADB
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RawDeviceInfo {
    pub serial: String,
    pub state: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub product: String,
}

impl RawDeviceInfo {
    pub fn is_connected(&self) -> bool {
        self.state == "device"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_connected() {
        let device = RawDeviceInfo {
            serial: "123".into(),
            state: "device".into(),
            ..Default::default()
        };
        assert!(device.is_connected());

        let offline = RawDeviceInfo {
            serial: "123".into(),
            state: "offline".into(),
            ..Default::default()
        };
        assert!(!offline.is_connected());
    }
}
