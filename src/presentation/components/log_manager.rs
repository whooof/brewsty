use std::collections::VecDeque;

const MAX_LOG_SIZE: usize = 200;

pub struct LogEntry {
    pub message: String,
    pub timestamp: std::time::SystemTime,
}

impl LogEntry {
    pub fn format_timestamp(&self) -> String {
        let timestamp = self.timestamp
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
}

impl LogManager {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::with_capacity(MAX_LOG_SIZE),
        }
    }

    pub fn push(&mut self, message: String) {
        if self.logs.len() >= MAX_LOG_SIZE {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            message,
            timestamp: std::time::SystemTime::now(),
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
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}
