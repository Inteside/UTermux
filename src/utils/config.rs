use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AppConfig {
    pub thread: usize,
    pub thread_num: usize,
    pub send_num: usize,
}

impl AppConfig {
    pub fn from_ini(path: &str) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read config file");
        let mut map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim(), v.trim());
            }
        }
        Self {
            thread: map.get("thread").and_then(|v| v.parse().ok()).unwrap_or(10),
            thread_num: map.get("thread_num").and_then(|v| v.parse().ok()).unwrap_or(2),
            send_num: map.get("send_num").and_then(|v| v.parse().ok()).unwrap_or(1),
        }
    }
}