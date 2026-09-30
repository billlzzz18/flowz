use std::collections::BTreeMap;

use decider::{Decider, DecisionQuery, Question, SystemOneClient};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};

async fn spawn_mock_server(
    status: &str,
    response_body: &'static str,
) -> (String, oneshot::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (request_tx, request_rx) = oneshot::channel();
    let status = status.to_string();

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut chunk = [0_u8; 2048];
        loop {
            let count = stream.read(&mut chunk).await.unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..count]);

            if let Some(header_end) = find_header_end(&request) {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
        }

        let _ = request_tx.send(String::from_utf8_lossy(&request).into_owned());
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });

    (format!("http://{address}"), request_rx)
}

fn find_header_end(request: &[u8]) -> Option<usize> {
    request.windows(4).position(|window| window == b"\r\n\r\n")
}

fn sample_query() -> DecisionQuery {
    DecisionQuery {
        state: json!({"text": "Synthetic support ticket: duplicate charge; asks if refund is available."}),
        questions: BTreeMap::from([
            (
                "route".to_string(),
                Question::Choice {
                    instructions: "Which route best matches this synthetic ticket?".to_string(),
                    criteria: BTreeMap::from([
                        ("billing".to_string(), Some("Payment or duplicate charge".to_string())),
                        ("other".to_string(), None),
                    ]),
                },
            ),
            (
                "eligible".to_string(),
                Question::Noul {
                    instructions: "Does the ticket explicitly request a refund?".to_string(),
                    criteria: None,
                },
            ),
            (
                "severity".to_string(),
                Question::Score {
                    instructions: "Score the urgency from low to high.".to_string(),
                    criteria: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
            ),
        ]),
        keep_alive: None,
    }
}

#[tokio::test]
async fn posts_all_question_types_and_decodes_answers() {
    let body = r#"{
        "answers": {
            "route": {"choice":"billing","probabilities":{"billing":0.91,"other":0.09},"confidence":0.91},
            "eligible": {"noul":0.24},
            "severity": {"score":1.2,"legend":["low","medium","high"],"probabilities":[0.1,0.6,0.3],"confidence":0.6}
        },
        "usage": {"input_tokens": 23}
    }"#;
    let (base_url, request_rx) = spawn_mock_server("200 OK", body).await;
    let client =
        SystemOneClient::new(&base_url, "tev1:4b", Some("mock-secret".to_string())).unwrap();

    let response = client.decide(sample_query()).await.unwrap();
    assert_eq!(response.answers["route"].choice.as_deref(), Some("billing"));
    assert_eq!(response.answers["eligible"].noul, Some(0.24));
    assert_eq!(response.answers["severity"].score, Some(1.2));
    assert_eq!(response.answers["severity"].confidence, Some(0.6));
    assert_eq!(response.usage, Some(json!({"input_tokens": 23})));

    let raw_request = request_rx.await.unwrap();
    assert!(raw_request.starts_with("POST /v1/systemone HTTP/1.1\r\n"));
    assert!(
        raw_request
            .to_ascii_lowercase()
            .contains("authorization: bearer mock-secret")
    );
    let request_body = raw_request.split_once("\r\n\r\n").unwrap().1;
    let request_json: Value = serde_json::from_str(request_body).unwrap();
    assert_eq!(request_json["model"], "tev1:4b");
    assert_eq!(request_json["questions"]["route"]["type"], "choice");
    assert_eq!(request_json["questions"]["eligible"]["type"], "noul");
    assert_eq!(request_json["questions"]["severity"]["type"], "score");
    assert!(!request_body.contains("mock-secret"));
}

#[tokio::test]
async fn accepts_full_endpoint_url_without_duplicating_path() {
    let (base_url, _) =
        spawn_mock_server("200 OK", r#"{"answers":{"eligible":{"noul":0.8}}}"#).await;
    let client =
        SystemOneClient::new(format!("{base_url}/v1/systemone/"), "test-model", None).unwrap();
    assert_eq!(client.endpoint(), format!("{base_url}/v1/systemone"));
}

#[tokio::test]
async fn omits_authorization_when_no_api_key_is_configured() {
    let (base_url, request_rx) =
        spawn_mock_server("200 OK", r#"{"answers":{"eligible":{"noul":0.8}}}"#).await;
    let client = SystemOneClient::new(&base_url, "tev1:4b", None).unwrap();
    client
        .decide(DecisionQuery {
            state: json!({"text": "synthetic"}),
            questions: BTreeMap::from([(
                "eligible".to_string(),
                Question::Noul {
                    instructions: "Is this synthetic?".to_string(),
                    criteria: None,
                },
            )]),
            keep_alive: None,
        })
        .await
        .unwrap();
    let raw_request = request_rx.await.unwrap();
    assert!(!raw_request.to_ascii_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn reports_http_status_without_echoing_response_body() {
    let (base_url, _) = spawn_mock_server("403 Forbidden", "private error payload").await;
    let client = SystemOneClient::new(&base_url, "jev", Some("mock-secret".to_string())).unwrap();
    let error = client.decide(sample_query()).await.unwrap_err();
    assert!(matches!(error, decider::DeciderError::HttpStatus { status: 403 }));
    assert!(!error.to_string().contains("private error payload"));
    assert!(!error.to_string().contains("mock-secret"));
}
