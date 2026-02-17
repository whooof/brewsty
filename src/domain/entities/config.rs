use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub auto_update_check: bool,
    pub confirm_before_actions: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            auto_update_check: true,
            confirm_before_actions: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemeMode::System);
        assert!(config.auto_update_check);
        assert!(config.confirm_before_actions);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let config = AppConfig {
            theme: ThemeMode::Dark,
            auto_update_check: false,
            confirm_before_actions: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, ThemeMode::Dark);
        assert!(!deserialized.auto_update_check);
        assert!(deserialized.confirm_before_actions);
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{"theme":"Light","auto_update_check":true,"confirm_before_actions":false}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme, ThemeMode::Light);
        assert!(config.auto_update_check);
        assert!(!config.confirm_before_actions);
    }
}
