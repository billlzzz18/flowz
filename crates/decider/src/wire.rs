use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::DeciderError;

/// A typed question sent to a System One-compatible endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice {
        instructions: Value,
        /// JSON description for each option; `null` leaves the key self-describing.
        criteria: BTreeMap<String, Value>,
    },
    Noul {
        instructions: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<Value>,
    },
    Score {
        instructions: Value,
        /// Ordered JSON descriptions from the lowest to the highest score level.
        criteria: Vec<Value>,
    },
}

impl Question {
    fn validate(&self, name: &str) -> Result<(), DeciderError> {
        match self {
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
                if criteria
                    .values()
                    .any(|value| !is_valid_criterion(value, true))
                {
                    return Err(DeciderError::InvalidRequest(format!(
                        "choice question {name:?} has an unsupported criterion value"
                    )));
                }
                validate_instructions(name, instructions)?;
            }
            Self::Noul { instructions, criteria } => {
                if let Some(criteria) = criteria {
                    let Value::Object(map) = criteria else {
                        return Err(DeciderError::InvalidRequest(format!(
                            "noul question {name:?} criteria must be an object"
                        )));
                    };
                    if map.values().any(|value| !is_valid_criterion(value, true)) {
                        return Err(DeciderError::InvalidRequest(format!(
                            "noul question {name:?} has an unsupported criterion value"
                        )));
                    }
                }
                validate_instructions(name, instructions)?;
            }
            Self::Score { instructions, criteria } => {
                if criteria.len() < 2 {
                    return Err(DeciderError::InvalidRequest(format!(
                        "score question {name:?} must have at least two levels"
                    )));
                }
                if criteria
                    .iter()
                    .any(|level| !is_valid_criterion(level, false))
                {
                    return Err(DeciderError::InvalidRequest(format!(
                        "score question {name:?} has an unsupported level description"
                    )));
                }
                validate_instructions(name, instructions)?;
            }
        }
        Ok(())
    }
}

fn validate_instructions(name: &str, instructions: &Value) -> Result<(), DeciderError> {
    let valid = match instructions {
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    };
    if !valid {
        return Err(DeciderError::InvalidRequest(format!(
            "question {name:?} has invalid instructions"
        )));
    }
    Ok(())
}

fn is_valid_criterion(value: &Value, allow_null: bool) -> bool {
    match value {
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(_) | Value::Object(_) => true,
        Value::Null => allow_null,
        Value::Bool(_) | Value::Number(_) => false,
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
        let is_empty_state = match &self.state {
            Value::Null => true,
            Value::String(text) => text.trim().is_empty(),
            Value::Array(items) => items.is_empty(),
            Value::Object(fields) => fields.is_empty(),
            _ => false,
        };
        if is_empty_state {
            return Err(DeciderError::InvalidRequest("state must not be empty".to_string()));
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
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
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
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AnswerError {
    #[error("answer is missing the `{0}` value")]
    MissingValue(&'static str),
}

impl Answer {
    pub fn choice_value(&self) -> Result<&str, AnswerError> {
        self.choice
            .as_deref()
            .ok_or(AnswerError::MissingValue("choice"))
    }

    pub fn noul_probability(&self) -> Result<f64, AnswerError> {
        self.noul.ok_or(AnswerError::MissingValue("noul"))
    }

    pub fn score_value(&self) -> Result<f64, AnswerError> {
        self.score.ok_or(AnswerError::MissingValue("score"))
    }
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
                    instructions: json!({"question": "Select a route", "context": ["Use the ticket text"]}),
                    criteria: BTreeMap::from([
                        ("billing".to_string(), json!("Payment issue")),
                        ("structured".to_string(), json!({"signals": ["invoice", "charge"]})),
                        ("other".to_string(), Value::Null),
                    ]),
                },
            ),
            (
                "eligible".to_string(),
                Question::Noul {
                    instructions: json!("Is it eligible?"),
                    criteria: Some(json!({"yes": "eligible", "no": "not eligible"})),
                },
            ),
            (
                "severity".to_string(),
                Question::Score {
                    instructions: json!("Rate severity"),
                    criteria: vec![json!("low"), json!("medium"), json!("high")],
                },
            ),
        ]);
        let value = serde_json::to_value(questions).unwrap();
        assert_eq!(value["route"]["type"], "choice");
        assert_eq!(value["route"]["criteria"]["other"], Value::Null);
        assert_eq!(value["route"]["criteria"]["structured"]["signals"][0], "invoice");
        assert_eq!(value["eligible"]["type"], "noul");
        assert_eq!(value["eligible"]["criteria"]["yes"], "eligible");
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
                    instructions: json!("Check"),
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
                    instructions: json!("Select"),
                    criteria: BTreeMap::from([("one".to_string(), Value::Null)]),
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
                    instructions: json!("Rate"),
                    criteria: vec![json!("only")],
                },
            )]),
            keep_alive: None,
        };
        assert!(score.validate().is_err());
    }

    #[test]
    fn answer_accessors_report_wrong_answer_shapes() {
        let answer = Answer {
            choice: Some("billing".to_string()),
            ..Default::default()
        };
        assert_eq!(answer.choice_value().unwrap(), "billing");
        assert_eq!(answer.noul_probability(), Err(AnswerError::MissingValue("noul")));
        assert_eq!(answer.score_value(), Err(AnswerError::MissingValue("score")));
    }

    #[test]
    fn rejects_scalar_instructions_and_unsupported_criterion_values() {
        let invalid_instruction = DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::from([(
                "q".to_string(),
                Question::Noul {
                    instructions: json!(true),
                    criteria: None,
                },
            )]),
            keep_alive: None,
        };
        assert!(invalid_instruction.validate().is_err());

        let invalid_criterion = DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::from([(
                "route".to_string(),
                Question::Choice {
                    instructions: json!("Select"),
                    criteria: BTreeMap::from([
                        ("a".to_string(), json!(1)),
                        ("b".to_string(), json!("valid")),
                    ]),
                },
            )]),
            keep_alive: None,
        };
        assert!(invalid_criterion.validate().is_err());
    }

    #[test]
    fn rejects_empty_state_variants() {
        for empty_state in [
            Value::Null,
            json!(""),
            json!("   "),
            json!([]),
            json!({}),
        ] {
            let query = DecisionQuery {
                state: empty_state,
                questions: BTreeMap::from([(
                    "q".to_string(),
                    Question::Noul {
                        instructions: json!("Check"),
                        criteria: None,
                    },
                )]),
                keep_alive: None,
            };
            let err = query.validate().unwrap_err();
            assert!(matches!(err, DeciderError::InvalidRequest(msg) if msg == "state must not be empty"));
        }
    }

    #[test]
    fn validates_noul_criteria_values() {
        let valid = DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::from([(
                "q".to_string(),
                Question::Noul {
                    instructions: json!("Check"),
                    criteria: Some(json!({"yes": "eligible", "no": null, "extra": ["details"]})),
                },
            )]),
            keep_alive: None,
        };
        assert!(valid.validate().is_ok());

        let invalid = DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::from([(
                "q".to_string(),
                Question::Noul {
                    instructions: json!("Check"),
                    criteria: Some(json!({"yes": 123})),
                },
            )]),
            keep_alive: None,
        };
        let err = invalid.validate().unwrap_err();
        assert!(matches!(err, DeciderError::InvalidRequest(msg) if msg.contains("has an unsupported criterion value")));
    }

    #[test]
    fn preserves_extra_unrecognized_fields_in_answer() {
        let json_str = r#"{
            "choice": "billing",
            "provider_custom_field": "custom_val",
            "debug_score": 42
        }"#;
        let answer: Answer = serde_json::from_str(json_str).unwrap();
        assert_eq!(answer.choice_value().unwrap(), "billing");
        assert_eq!(answer.extra.get("provider_custom_field"), Some(&json!("custom_val")));
        assert_eq!(answer.extra.get("debug_score"), Some(&json!(42)));
    }
}
