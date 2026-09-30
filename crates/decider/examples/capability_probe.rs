use std::{collections::BTreeMap, env, process};

use decider::{Decider, DecisionQuery, Question, SystemOneClient};
use serde_json::json;

fn shared_query() -> DecisionQuery {
    DecisionQuery {
        state: json!({
            "ticket": "Synthetic example only: customer reports a duplicate card charge and asks whether a refund is available."
        }),
        questions: BTreeMap::from([
            (
                "route".to_string(),
                Question::Choice {
                    instructions: "Which category best matches the ticket?".to_string(),
                    criteria: BTreeMap::from([
                        ("billing".to_string(), Some("Payment or duplicate charge".to_string())),
                        (
                            "refund".to_string(),
                            Some("Explicit request to return money".to_string()),
                        ),
                        ("other".to_string(), None),
                    ]),
                },
            ),
            (
                "explicit_refund_request".to_string(),
                Question::Noul {
                    instructions: "Does the customer explicitly request a refund?".to_string(),
                    criteria: None,
                },
            ),
            (
                "urgency".to_string(),
                Question::Score {
                    instructions: "Score the urgency of this ticket from low to high.".to_string(),
                    criteria: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
            ),
        ]),
        keep_alive: None,
    }
}

async fn probe(label: &str, prefix: &str) -> Option<bool> {
    let base_url = match env::var(format!("{prefix}_BASE_URL")) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            println!("SKIP {label}: {prefix}_BASE_URL is not configured");
            return None;
        }
    };
    let model = match env::var(format!("{prefix}_MODEL")) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            println!("SKIP {label}: {prefix}_MODEL is not configured");
            return None;
        }
    };
    let api_key = env::var(format!("{prefix}_API_KEY")).ok();
    let client = match SystemOneClient::new(base_url, model, api_key) {
        Ok(client) => client,
        Err(error) => {
            println!("FAIL {label}: {error}");
            return Some(false);
        }
    };

    match client.decide(shared_query()).await {
        Ok(response) => {
            let choice_ok = response
                .answers
                .get("route")
                .is_some_and(|answer| answer.choice.is_some());
            let noul_ok = response
                .answers
                .get("explicit_refund_request")
                .is_some_and(|answer| answer.noul.is_some());
            let score_ok = response
                .answers
                .get("urgency")
                .is_some_and(|answer| answer.score.is_some());
            println!(
                "{label}: Choice={} Noul={} Score={}",
                if choice_ok { "PASS" } else { "FAIL" },
                if noul_ok { "PASS" } else { "FAIL" },
                if score_ok { "PASS" } else { "FAIL" },
            );
            for name in ["route", "explicit_refund_request", "urgency"] {
                if let Some(answer) = response.answers.get(name) {
                    println!(
                        "  {name}: choice={:?} noul={:?} score={:?} confidence={:?}",
                        answer.choice, answer.noul, answer.score, answer.confidence
                    );
                } else {
                    println!("  {name}: missing answer");
                }
            }
            Some(choice_ok && noul_ok && score_ok)
        }
        Err(error) => {
            println!("FAIL {label}: {error}");
            Some(false)
        }
    }
}

#[tokio::main]
async fn main() {
    let mut configured = 0;
    let mut all_passed = true;
    for (label, prefix) in [("Jev", "JEV"), ("Laya", "LAYA"), ("tev1", "TEV1")] {
        if let Some(passed) = probe(label, prefix).await {
            configured += 1;
            all_passed &= passed;
        }
    }
    if configured == 0 {
        eprintln!(
            "No live backend was probed. Configure one or more endpoint/model environment variables first."
        );
        process::exit(2);
    }
    if !all_passed {
        process::exit(1);
    }
}
