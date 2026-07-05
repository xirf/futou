use std::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub runtime: String,
    pub level: String,
    pub message: String,
}

pub struct OperationLog {
    entries: RwLock<Vec<LogEntry>>,
    max: usize,
}

impl OperationLog {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            max: 500,
        }
    }

    pub fn push(&self, runtime: &str, level: &str, message: String) {
        let mut entries = self.entries.write().unwrap();
        let timestamp = chrono::Utc::now().format("%H:%M:%S").to_string();
        entries.push(LogEntry {
            timestamp,
            runtime: runtime.to_string(),
            level: level.to_string(),
            message,
        });
        if entries.len() > self.max {
            entries.drain(..100);
        }
    }

    pub fn for_runtime(&self, runtime: &str) -> Vec<LogEntry> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.runtime == runtime || e.runtime.is_empty())
            .cloned()
            .collect()
    }
}
