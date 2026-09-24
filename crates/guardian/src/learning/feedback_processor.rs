use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub ai_output: String,
    pub feedback_type: FeedbackType,
    pub correction: Option<String>,
    pub severity: String,
    pub tags: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeedbackType {
    Incorrect,
    Incomplete,
    Inefficient,
    Unsafe,
    Misunderstood,
    Hallucination,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub description: String,
    pub example: String,
    pub count: usize,
}
pub struct FeedbackProcessor {
    history: Vec<FeedbackEntry>,
    pub patterns: HashMap<String, Pattern>,
}
impl Default for FeedbackProcessor {
    fn default() -> Self {
        Self::new()
    }
}
impl FeedbackProcessor {
    pub fn new() -> Self {
        Self {
            history: vec![],
            patterns: HashMap::new(),
        }
    }
    pub fn process_feedback(&mut self, e: FeedbackEntry) {
        let key = e.tags.join(",");
        let p = self.patterns.entry(key).or_insert(Pattern {
            description: e.user_input.clone(),
            example: e.correction.clone().unwrap_or(e.ai_output.clone()),
            count: 0,
        });
        p.count += 1;
        self.history.push(e)
    }
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
    pub fn get_recommendations(&self, input: &str) -> Vec<String> {
        self.patterns
            .iter()
            .filter(|(k, _)| {
                let input = input.to_ascii_lowercase();
                k.split(',')
                    .any(|tag| input.contains(&tag.to_ascii_lowercase()))
            })
            .map(|(_, p)| format!("Based on feedback: {}", p.example))
            .collect()
    }
}
