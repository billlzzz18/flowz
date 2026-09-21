// Simple test worker that reads JSON from stdin and writes JSON to stdout
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct WorkerRequest {
    item_id: String,
    prompt: String,
    brief: String,
    output_schema: serde_json::Value,
    requirements: String,
    source_requirements: String,
    input_files: Vec<InputFile>,
    sandbox: String,
    effort_level: String,
}

#[derive(Debug, Deserialize)]
struct InputFile {
    name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct WorkerResponse {
    item_id: String,
    ok: bool,
    value: Option<serde_json::Value>,
    files: Vec<String>,
    error: Option<String>,
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    if let Some(Ok(line)) = lines.next() {
        let request: WorkerRequest = serde_json::from_str(&line).unwrap();
        
        // Simple test response
        let response = WorkerResponse {
            item_id: request.item_id,
            ok: true,
            value: Some(serde_json::json!({
                "summary": format!("Test response for: {}", request.brief),
                "sources": ["test-source"]
            })),
            files: vec![],
            error: None,
        };
        
        let mut stdout = io::stdout();
        writeln!(stdout, "{}", serde_json::to_string(&response).unwrap())?;
        stdout.flush()?;
    }
    
    Ok(())
}