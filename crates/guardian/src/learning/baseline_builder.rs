use crate::core::metrics::AIMetrics;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub struct CommitMetrics {
    pub slop_score: f64,
    pub quality_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBaseline {
    pub team_id: String,
    pub avg_slop_score: f64,
    pub avg_quality_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub slop_delta: f64,
    pub quality_delta: f64,
    pub status: String,
}
pub struct BaselineBuilder;
impl BaselineBuilder {
    pub fn build_from_history(c: &[CommitMetrics]) -> TeamBaseline {
        let n = c.len().max(1) as f64;
        TeamBaseline {
            team_id: "team_001".into(),
            avg_slop_score: c.iter().map(|x| x.slop_score).sum::<f64>() / n,
            avg_quality_score: c.iter().map(|x| x.quality_score).sum::<f64>() / n,
        }
    }
    pub fn compare_against_baseline(cur: &AIMetrics, b: &TeamBaseline) -> ComparisonResult {
        let q = cur.quality_score - b.avg_quality_score;
        ComparisonResult {
            slop_delta: cur.metrics.slop_score - b.avg_slop_score,
            quality_delta: q,
            status: if q >= 0.0 {
                "Above Baseline".into()
            } else {
                "Below Baseline".into()
            },
        }
    }
}
