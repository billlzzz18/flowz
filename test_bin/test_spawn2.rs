use tokio::process::Command;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("./worker")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let payload = br#"{"job_id": "test", "item_id": "1", "prompt": "test", "brief": "rust async patterns", "schema": {}, "input_files": [], "sandbox": "isolated", "effort_level": "lite"}\n"#;
    if let Some(mut stdin) = child.stdin.take() {
        println!("Writing payload... ({} bytes)", payload.len());
        stdin.write_all(payload).await?;
        println!("Payload written");
        stdin.shutdown().await?;
        println!("Stdin shutdown");
    }

    let output = child.wait_with_output().await?;
    println!("Exit status: {:?}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}
