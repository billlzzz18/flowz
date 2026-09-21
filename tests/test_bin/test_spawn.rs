use tokio::process::Command;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let payload = b"hello\n";
    if let Some(mut stdin) = child.stdin.take() {
        println!("Writing payload...");
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
