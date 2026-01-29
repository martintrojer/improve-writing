use anyhow::{Context, Result};
use tokio::process::Command;

/// Type text using wtype (Wayland)
pub async fn type_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    Command::new("wtype")
        .arg(text)
        .status()
        .await
        .context("Failed to type text (is wtype installed?)")?;

    Ok(())
}

/// Copy text to clipboard using wl-copy
pub async fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command as TokioCommand;

    let mut child = TokioCommand::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to run wl-copy (is wl-clipboard installed?)")?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(text.as_bytes()).await?;
    }

    child.wait().await.context("wl-copy failed")?;
    Ok(())
}

/// Get primary selection (highlighted text) using wl-paste
pub async fn get_primary_selection() -> Result<String> {
    let output = Command::new("wl-paste")
        .arg("--primary")
        .output()
        .await
        .context("Failed to get primary selection (is wl-clipboard installed?)")?;

    if !output.status.success() {
        anyhow::bail!("wl-paste failed: {:?}", output.status);
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}
