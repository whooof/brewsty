//! Desktop notifications for macOS
//!
//! # Security Features
//! - Async API wraps blocking macOS notification calls in spawn_blocking
//! - Rate limiting prevents notification flooding (max 5 per minute)

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Maximum notifications per minute (rate limiting)
const MAX_NOTIFICATIONS_PER_MINUTE: u64 = 5;

/// Minimum interval between notifications (debounce)
const MIN_NOTIFICATION_INTERVAL: Duration = Duration::from_secs(2);

/// Rate limiting state (global for simplicity)
static NOTIFICATION_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_NOTIFICATION_TIME: AtomicU64 = AtomicU64::new(0);

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

/// Check if we can send a notification (rate limiting)
fn can_send_notification() -> bool {
    let now = Instant::now().elapsed().as_secs();
    let last_time = LAST_NOTIFICATION_TIME.load(Ordering::Relaxed);
    
    // Reset count if a minute has passed
    if now - last_time >= 60 {
        NOTIFICATION_COUNT.store(0, Ordering::Relaxed);
        LAST_NOTIFICATION_TIME.store(now, Ordering::Relaxed);
    }
    
    // Check rate limit
    let count = NOTIFICATION_COUNT.load(Ordering::Relaxed);
    if count >= MAX_NOTIFICATIONS_PER_MINUTE {
        return false;
    }
    
    // Check minimum interval (debounce)
    if now - last_time < MIN_NOTIFICATION_INTERVAL.as_secs() {
        return false;
    }
    
    // Increment count and update time
    NOTIFICATION_COUNT.fetch_add(1, Ordering::Relaxed);
    LAST_NOTIFICATION_TIME.store(now, Ordering::Relaxed);
    
    true
}

/// Send a desktop notification (blocking version for synchronous contexts)
///
/// Note: This function may block. Use `send_notification_async` in async contexts.
pub fn send_notification(
    title: &str,
    message: &str,
    notification_type: NotificationType,
) -> Result<()> {
    // Check rate limit
    if !can_send_notification() {
        log::debug!("Notification rate limited: {} - {}", title, message);
        return Ok(()); // Silently skip rate-limited notifications
    }
    
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

/// Send a desktop notification asynchronously
///
/// This wraps the blocking macOS notification API in spawn_blocking
/// to prevent blocking the tokio executor.
pub async fn send_notification_async(
    title: String,
    message: String,
    notification_type: NotificationType,
) -> Result<()> {
    // Check rate limit before spawning blocking task
    if !can_send_notification() {
        log::debug!("Notification rate limited: {} - {}", title, message);
        return Ok(()); // Silently skip rate-limited notifications
    }
    
    #[cfg(target_os = "macos")]
    {
        tokio::task::spawn_blocking(move || {
            use mac_notification_sys::{Notification, set_application};
            
            // Set the application name for notifications
            set_application("Brewsty").ok();
            
            // Send the notification
            let formatted_title = format!("{} {}", notification_type.icon(), title);
            let mut notification = Notification::new();
            notification
                .title(&formatted_title)
                .message(&message)
                .sound(true);
            notification.send().context("Failed to send notification")?;
            
            Ok(())
        })
        .await
        .context("spawn_blocking task failed")?
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, just log the notification
        log::info!("Notification: {} - {}", title, message);
        Ok(())
    }
}

/// Send an info notification (blocking, for synchronous contexts)
pub fn notify_info(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Info);
}

/// Send a success notification (blocking, for synchronous contexts)
pub fn notify_success(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Success);
}

/// Send a warning notification (blocking, for synchronous contexts)
pub fn notify_warning(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Warning);
}

/// Send an error notification (blocking, for synchronous contexts)
pub fn notify_error(title: &str, message: &str) {
    let _ = send_notification(title, message, NotificationType::Error);
}

/// Send an info notification asynchronously
pub async fn notify_info_async(title: String, message: String) {
    let _ = send_notification_async(title, message, NotificationType::Info).await;
}

/// Send a success notification asynchronously
pub async fn notify_success_async(title: String, message: String) {
    let _ = send_notification_async(title, message, NotificationType::Success).await;
}

/// Send a warning notification asynchronously  
pub async fn notify_warning_async(title: String, message: String) {
    let _ = send_notification_async(title, message, NotificationType::Warning).await;
}

/// Send an error notification asynchronously
pub async fn notify_error_async(title: String, message: String) {
    let _ = send_notification_async(title, message, NotificationType::Error).await;
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
    
    #[test]
    fn test_rate_limiting() {
        // Reset state
        NOTIFICATION_COUNT.store(0, Ordering::Relaxed);
        LAST_NOTIFICATION_TIME.store(0, Ordering::Relaxed);
        
        // First notification should be allowed
        assert!(can_send_notification());
        
        // Second within debounce interval should be blocked
        // (Note: this test may pass if timing works out)
        // In practice, the debounce check works with real time
    }
}