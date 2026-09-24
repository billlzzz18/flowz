use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPractice {
    pub id: String,
    pub name: String,
    pub description: String,
    pub example_code: String,
    pub frequency: usize,
    pub effectiveness_score: f64,
    pub tags: Vec<String>,
    pub last_used: String,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PatternLibrary {
    pub best_practices: HashMap<String, BestPractice>,
}
impl PatternLibrary {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_best_practice(&mut self, p: BestPractice) {
        self.best_practices.insert(p.id.clone(), p);
    }
    pub fn get_top_practices(&self, limit: usize) -> Vec<&BestPractice> {
        let mut v = self.best_practices.values().collect::<Vec<_>>();
        v.sort_by(|a, b| b.effectiveness_score.total_cmp(&a.effectiveness_score));
        v.into_iter().take(limit).collect()
    }
}
