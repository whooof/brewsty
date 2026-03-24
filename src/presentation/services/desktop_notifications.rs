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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_icon() {
        assert_eq!(NotificationType::Info.icon(), "ℹ️");
        assert_eq!(NotificationType::Success.icon(), "✅");
        assert_eq!(NotificationType::Warning.icon(), "⚠️");
        assert_eq!(NotificationType::Error.icon(), "❌");
    }
}
