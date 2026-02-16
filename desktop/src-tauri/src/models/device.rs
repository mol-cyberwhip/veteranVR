use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, specta::Type)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub product: String,
}

impl DeviceInfo {
    pub fn is_connected(&self) -> bool {
        self.state == "device"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_connected() {
        let device = DeviceInfo {
            serial: "123".into(),
            state: "device".into(),
            ..Default::default()
        };
        assert!(device.is_connected());

        let offline = DeviceInfo {
            serial: "123".into(),
            state: "offline".into(),
            ..Default::default()
        };
        assert!(!offline.is_connected());
    }
}
