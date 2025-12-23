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
