use std::collections::VecDeque;

const MAX_LOG_SIZE: usize = 200;
const DISPLAY_LOG_SIZE: usize = 20;

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

    pub fn recent(&self) -> impl Iterator<Item = &String> {
        self.logs
            .iter()
            .rev()
            .take(DISPLAY_LOG_SIZE)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}
