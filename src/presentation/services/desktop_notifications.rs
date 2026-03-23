//! Desktop notifications for macOS

use anyhow::{Context, Result};

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationType {
    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::Info => "ℹ️",
            NotificationType::Success => "✅",
            NotificationType::Warning => "⚠️",
            NotificationType::Error => "❌",
        }
    }
}

/// Send a desktop notification
pub fn send_notification(
    title: &str,
    message: &str,
    notification_type: NotificationType,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use mac_notification_sys::{Notification, set_application};

        // Set the application name for notifications
        set_application("Brewsty").ok();

        // Send the notification
        let formatted_title = format!("{} {}", notification_type.icon(), title);
        let mut notification = Notification::new();
        notification
            .title(&formatted_title)
            .message(message)
            .sound(true);
        notification.send().context("Failed to send notification")?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, just log the notification
        log::info!("Notification: {} - {}", title, message);
    }

    Ok(())
}

/// Send an info notification
pub fn notify_info(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Info);
}

/// Send a success notification
pub fn notify_success(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Success);
}

/// Send a warning notification
pub fn notify_warning(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Warning);
}

/// Send an error notification
pub fn notify_error(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Error);
}

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub enabled: bool,
    #[allow(dead_code)]
    pub show_on_install: bool,
    #[allow(dead_code)]
    pub show_on_uninstall: bool,
    #[allow(dead_code)]
    pub show_on_update: bool,
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

impl NotificationConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn should_notify(&self, notification_type: NotificationType) -> bool {
        if !self.enabled {
            return false;
        }

        match notification_type {
            NotificationType::Info => true,
            NotificationType::Success => true,
            NotificationType::Warning => true,
            NotificationType::Error => self.show_on_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.show_on_install);
        assert!(config.show_on_uninstall);
        assert!(config.show_on_update);
        assert!(config.show_on_error);
    }

    #[test]
    fn test_notification_config_should_notify() {
        let mut config = NotificationConfig::default();

        // All notifications enabled
        assert!(config.should_notify(NotificationType::Info));
        assert!(config.should_notify(NotificationType::Success));
        assert!(config.should_notify(NotificationType::Warning));
        assert!(config.should_notify(NotificationType::Error));

        // Disable all
        config.enabled = false;
        assert!(!config.should_notify(NotificationType::Info));
        assert!(!config.should_notify(NotificationType::Success));
        assert!(!config.should_notify(NotificationType::Warning));
        assert!(!config.should_notify(NotificationType::Error));

        // Re-enable but disable errors
        config.enabled = true;
        config.show_on_error = false;
        assert!(config.should_notify(NotificationType::Info));
        assert!(!config.should_notify(NotificationType::Error));
    }

    #[test]
    fn test_notification_type_icon() {
        assert_eq!(NotificationType::Info.icon(), "ℹ️");
        assert_eq!(NotificationType::Success.icon(), "✅");
        assert_eq!(NotificationType::Warning.icon(), "⚠️");
        assert_eq!(NotificationType::Error.icon(), "❌");
    }
}
