use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_debounce_delay() -> u64 {
    2000 // 2 seconds
}

fn default_notification_config() -> NotificationConfig {
    NotificationConfig::default()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub auto_update_check: bool,
    pub confirm_before_actions: bool,

    #[serde(default = "default_true")]
    pub search_debounce_enabled: bool,

    #[serde(default = "default_debounce_delay")]
    pub search_debounce_delay: u64, // in milliseconds

    #[serde(default = "default_notification_config")]
    pub notifications: NotificationConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub show_on_install: bool,
    #[serde(default = "default_true")]
    pub show_on_uninstall: bool,
    #[serde(default = "default_true")]
    pub show_on_update: bool,
    #[serde(default = "default_true")]
    pub show_on_error: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_on_install: true,
            show_on_uninstall: true,
            show_on_update: true,
            show_on_error: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            auto_update_check: true,
            confirm_before_actions: true,
            search_debounce_enabled: true,
            search_debounce_delay: 2000,
            notifications: NotificationConfig::default(),
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
        assert!(config.search_debounce_enabled);
        assert_eq!(config.search_debounce_delay, 2000);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let config = AppConfig {
            theme: ThemeMode::Dark,
            auto_update_check: false,
            confirm_before_actions: true,
            search_debounce_enabled: false,
            search_debounce_delay: 1000,
            notifications: NotificationConfig::default(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, ThemeMode::Dark);
        assert!(!deserialized.auto_update_check);
        assert!(deserialized.confirm_before_actions);
        assert!(!deserialized.search_debounce_enabled);
        assert_eq!(deserialized.search_debounce_delay, 1000);
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{"theme":"Light","auto_update_check":true,"confirm_before_actions":false}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme, ThemeMode::Light);
        assert!(config.auto_update_check);
        assert!(!config.confirm_before_actions);
        assert!(config.search_debounce_enabled);
        assert_eq!(config.search_debounce_delay, 2000);
        // notifications should use defaults
        assert!(config.notifications.enabled);
        assert!(config.notifications.show_on_install);
    }

    #[test]
    fn notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.show_on_install);
        assert!(config.show_on_uninstall);
        assert!(config.show_on_update);
        assert!(config.show_on_error);
    }

    #[test]
    fn notification_config_serialization() {
        let config = NotificationConfig {
            enabled: false,
            show_on_install: true,
            show_on_uninstall: false,
            show_on_update: true,
            show_on_error: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: NotificationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, false);
        assert_eq!(deserialized.show_on_install, true);
        assert_eq!(deserialized.show_on_uninstall, false);
        assert_eq!(deserialized.show_on_update, true);
        assert_eq!(deserialized.show_on_error, false);
    }
}
