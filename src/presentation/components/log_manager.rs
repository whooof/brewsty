use std::collections::VecDeque;

const MAX_LOG_SIZE: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

pub struct LogEntry {
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub level: LogLevel,
}

impl LogEntry {
    pub fn format_timestamp(&self) -> String {
        let timestamp = self
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let hours = (timestamp.as_secs() / 3600) % 24;
        let minutes = (timestamp.as_secs() / 60) % 60;
        let seconds = timestamp.as_secs() % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub struct LogManager {
    logs: VecDeque<LogEntry>,
    visible_levels: std::collections::HashSet<LogLevel>,
}

impl LogManager {
    pub fn new() -> Self {
        let mut visible_levels = std::collections::HashSet::new();
        visible_levels.insert(LogLevel::Info);
        visible_levels.insert(LogLevel::Warn);
        visible_levels.insert(LogLevel::Error);
        Self {
            logs: VecDeque::with_capacity(MAX_LOG_SIZE),
            visible_levels,
        }
    }

    pub fn push(&mut self, message: String) {
        let level = message
            .split(']')
            .next()
            .and_then(|s| s.strip_prefix('['))
            .and_then(|level_str| LogLevel::from_str(level_str))
            .unwrap_or(LogLevel::Info);
        if self.logs.len() >= MAX_LOG_SIZE {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            message,
            timestamp: std::time::SystemTime::now(),
            level,
        });
    }

    pub fn extend(&mut self, messages: Vec<String>) {
        for message in messages {
            self.push(message);
        }
    }

    pub fn all_logs(&self) -> impl Iterator<Item = &LogEntry> {
        self.logs.iter()
    }

    pub fn filtered_logs(&self) -> impl Iterator<Item = &LogEntry> {
        self.logs
            .iter()
            .filter(move |entry| self.visible_levels.contains(&entry.level))
    }

    pub fn filtered_logs_reversed(&self) -> impl Iterator<Item = &LogEntry> {
        self.logs
            .iter()
            .rev()
            .filter(move |entry| self.visible_levels.contains(&entry.level))
    }

    pub fn set_level_visible(&mut self, level: LogLevel, visible: bool) {
        if visible {
            self.visible_levels.insert(level);
        } else {
            self.visible_levels.remove(&level);
        }
    }

    pub fn is_level_visible(&self, level: LogLevel) -> bool {
        self.visible_levels.contains(&level)
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}
