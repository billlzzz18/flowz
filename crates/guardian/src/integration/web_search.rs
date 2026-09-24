use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub claim: String,
    pub found_supporting_evidence: bool,
    pub sources: Vec<SearchResult>,
    pub confidence: f64,
}
