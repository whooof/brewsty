use std::borrow::Cow;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserMessage {
    pub title: String,
    pub body: String,
    pub severity: MessageSeverity,
    pub recovery_action: Option<String>,
    pub details: Option<String>,
}

impl UserMessage {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        severity: MessageSeverity,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            severity,
            recovery_action: None,
            details: None,
        }
    }

    pub fn with_recovery_action(mut self, action: impl Into<String>) -> Self {
        self.recovery_action = Some(action.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LoadState<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Error(AppError),
    Partial {
        data: T,
        warning: AppError,
    },
}

impl<T> LoadState<T> {
    #[allow(dead_code)]
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    #[allow(dead_code)]
    pub fn ready_data(&self) -> Option<&T> {
        match self {
            Self::Ready(data) => Some(data),
            Self::Partial { data, .. } => Some(data),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn error(&self) -> Option<&AppError> {
        match self {
            Self::Error(error) => Some(error),
            Self::Partial { warning, .. } => Some(warning),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OperationState {
    #[default]
    Idle,
    Running {
        kind: Cow<'static, str>,
        target: Option<String>,
    },
    Succeeded {
        message: String,
    },
    Failed {
        error: AppError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl CommandResult {
    pub fn succeeded(&self) -> bool {
        self.exit_code.unwrap_or_default() == 0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AppError {
    #[error("Homebrew executable is not available")]
    BrewMissing,
    #[error("Command failed: {command}")]
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    #[error("Password prompt was cancelled")]
    AuthCancelled,
    #[error("Incorrect administrator password")]
    AuthFailed,
    #[error("Parse error: {0}")]
    ParseFailed(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[allow(dead_code)]
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("{0}")]
    Unknown(String),
}

impl AppError {
    pub fn short_message(&self) -> String {
        match self {
            Self::BrewMissing => "Homebrew is not installed or not available in PATH.".to_string(),
            Self::CommandFailed { command, .. } => format!("Command failed: {command}"),
            Self::AuthCancelled => "Password prompt was cancelled.".to_string(),
            Self::AuthFailed => "Administrator password was rejected.".to_string(),
            Self::ParseFailed(message)
            | Self::Io(message)
            | Self::Config(message)
            | Self::Validation(message)
            | Self::Unknown(message) => message.clone(),
        }
    }

    pub fn details(&self) -> Option<String> {
        match self {
            Self::CommandFailed {
                exit_code, stderr, ..
            } => {
                let mut parts = Vec::new();
                if let Some(code) = exit_code {
                    parts.push(format!("exit code: {code}"));
                }
                if !stderr.trim().is_empty() {
                    parts.push(stderr.trim().to_string());
                }
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join("\n"))
                }
            }
            Self::ParseFailed(message)
            | Self::Io(message)
            | Self::Config(message)
            | Self::Validation(message)
            | Self::Unknown(message) => Some(message.clone()),
            Self::BrewMissing | Self::AuthCancelled | Self::AuthFailed => None,
        }
    }

    pub fn to_user_message(&self, title: impl Into<String>) -> UserMessage {
        let body = match self {
            Self::BrewMissing => {
                "Install Homebrew and ensure the `brew` command is available in PATH."
            }
            Self::AuthCancelled => "Run the action again if you still want to continue.",
            Self::AuthFailed => "Try the action again and enter a valid administrator password.",
            Self::Config(_) => "Fix the configuration or reset it, then retry.",
            Self::Validation(_) => "Correct the input and retry.",
            Self::ParseFailed(_) | Self::Io(_) | Self::CommandFailed { .. } | Self::Unknown(_) => {
                "Review the details and logs, then retry."
            }
        };

        let mut message = UserMessage::new(title, body, MessageSeverity::Error);
        message.details = self.details();
        message.recovery_action = Some("Retry".to_string());
        message
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        if let Some(app_error) = error.downcast_ref::<Self>() {
            return app_error.clone();
        }

        Self::classify_message(error.to_string())
    }

    pub fn classify_message(message: impl Into<String>) -> Self {
        let message = message.into();
        let lowered = message.to_lowercase();

        if lowered.contains("password prompt was cancelled")
            || lowered.contains("no password was provided")
            || lowered.contains("user canceled")
            || lowered.contains("user cancelled")
        {
            Self::AuthCancelled
        } else if lowered.contains("incorrect password")
            || lowered.contains("incorrect password attempt")
            || lowered.contains("sorry, try again")
        {
            Self::AuthFailed
        } else if lowered.contains("no such file or directory")
            && (lowered.contains("brew") || lowered.contains("homebrew"))
        {
            Self::BrewMissing
        } else {
            Self::Unknown(message)
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        if value.kind() == std::io::ErrorKind::NotFound {
            Self::BrewMissing
        } else {
            Self::Io(value.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::ParseFailed(value.to_string())
    }
}
