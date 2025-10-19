use std::collections::VecDeque;

const MAX_LOG_SIZE: usize = 200;

pub struct LogManager {
    logs: VecDeque<String>,
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
        self.logs.push_back(message);
    }

    pub fn extend(&mut self, messages: Vec<String>) {
        for message in messages {
            self.push(message);
        }
    }

    pub fn all_logs(&self) -> Vec<&String> {
        self.logs.iter().collect()
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}
