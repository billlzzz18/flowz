use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::DeciderError;

/// A typed question sent to a System One-compatible endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice {
        instructions: String,
        /// Maps each option key to an optional description. `null` lets the key describe itself.
        criteria: BTreeMap<String, Option<String>>,
    },
    Noul {
        instructions: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<BTreeMap<String, Option<String>>>,
    },
    Score {
        instructions: String,
        /// Ordered descriptions from the lowest to the highest score level.
        criteria: Vec<String>,
    },
}

impl Question {
    fn validate(&self, name: &str) -> Result<(), DeciderError> {
        let instructions = match self {
            Self::Choice { instructions, criteria } => {
                if criteria.len() < 2 {
                    return Err(DeciderError::InvalidRequest(format!(
                        "choice question {name:?} must have at least two options"
                    )));
                }
                if criteria.keys().any(|key| key.trim().is_empty()) {
                    return Err(DeciderError::InvalidRequest(format!(
                        "choice question {name:?} has an empty option key"
                    )));
                }
                instructions
            }
            Self::Noul { instructions, .. } => instructions,
            Self::Score { instructions, criteria } => {
                if criteria.len() < 2 {
                    return Err(DeciderError::InvalidRequest(format!(
                        "score question {name:?} must have at least two levels"
                    )));
                }
                if criteria.iter().any(|level| level.trim().is_empty()) {
                    return Err(DeciderError::InvalidRequest(format!(
                        "score question {name:?} has an empty level description"
                    )));
                }
                instructions
            }
        };

        if instructions.trim().is_empty() {
            return Err(DeciderError::InvalidRequest(format!(
                "question {name:?} has empty instructions"
            )));
        }
        Ok(())
    }
}

/// Provider-independent input. The model name is owned by `SystemOneClient`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionQuery {
    pub state: Value,
    pub questions: BTreeMap<String, Question>,
    /// Optional provider-specific keep-alive value (for example, Ollama's `"5m"`).
    pub keep_alive: Option<Value>,
}

impl DecisionQuery {
    pub fn validate(&self) -> Result<(), DeciderError> {
        if self.state.is_null() {
            return Err(DeciderError::InvalidRequest("state must not be null".to_string()));
        }
        if self.questions.is_empty() {
            return Err(DeciderError::InvalidRequest(
                "at least one question is required".to_string(),
            ));
        }
        if self.questions.keys().any(|name| name.trim().is_empty()) {
            return Err(DeciderError::InvalidRequest(
                "question names must not be empty".to_string(),
            ));
        }
        for (name, question) in &self.questions {
            question.validate(name)?;
        }
        Ok(())
    }
}

/// Exact HTTP request envelope sent to `/v1/systemone`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub model: String,
    pub state: Value,
    pub questions: BTreeMap<String, Question>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<Value>,
}

/// One named answer. Optional fields preserve the different answer shapes and
/// provider extensions without coercing probabilities or confidence values.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Answer {
    #[serde(default)]
    pub choice: Option<String>,
    #[serde(default)]
    pub noul: Option<f64>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub probabilities: Option<Value>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub legend: Option<Value>,
}

/// Response envelope returned by a System One-compatible endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecisionResponse {
    pub answers: BTreeMap<String, Answer>,
    #[serde(default)]
    pub usage: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_all_question_types_using_system_one_tags() {
        let questions = BTreeMap::from([
            (
                "route".to_string(),
                Question::Choice {
                    instructions: "Select a route".to_string(),
                    criteria: BTreeMap::from([
                        ("billing".to_string(), Some("Payment issue".to_string())),
                        ("other".to_string(), None),
                    ]),
                },
            ),
            (
                "eligible".to_string(),
                Question::Noul {
                    instructions: "Is it eligible?".to_string(),
                    criteria: None,
                },
            ),
            (
                "severity".to_string(),
                Question::Score {
                    instructions: "Rate severity".to_string(),
                    criteria: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
            ),
        ]);
        let value = serde_json::to_value(questions).unwrap();
        assert_eq!(value["route"]["type"], "choice");
        assert_eq!(value["route"]["criteria"]["other"], Value::Null);
        assert_eq!(value["eligible"]["type"], "noul");
        assert!(value["eligible"].get("criteria").is_none());
        assert_eq!(value["severity"]["type"], "score");
        assert_eq!(value["severity"]["criteria"], json!(["low", "medium", "high"]));
    }

    #[test]
    fn rejects_null_state_and_empty_question_sets() {
        let null_state = DecisionQuery {
            state: Value::Null,
            questions: BTreeMap::from([(
                "q".to_string(),
                Question::Noul {
                    instructions: "Check".to_string(),
                    criteria: None,
                },
            )]),
            keep_alive: None,
        };
        assert!(null_state.validate().is_err());

        let no_questions = DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::new(),
            keep_alive: None,
        };
        assert!(no_questions.validate().is_err());
    }

    #[test]
    fn rejects_single_option_choice_and_score() {
        let state = json!({"text": "synthetic"});
        let choice = DecisionQuery {
            state: state.clone(),
            questions: BTreeMap::from([(
                "route".to_string(),
                Question::Choice {
                    instructions: "Select".to_string(),
                    criteria: BTreeMap::from([("one".to_string(), None)]),
                },
            )]),
            keep_alive: None,
        };
        assert!(choice.validate().is_err());

        let score = DecisionQuery {
            state,
            questions: BTreeMap::from([(
                "severity".to_string(),
                Question::Score {
                    instructions: "Rate".to_string(),
                    criteria: vec!["only".to_string()],
                },
            )]),
            keep_alive: None,
        };
        assert!(score.validate().is_err());
    }
}
