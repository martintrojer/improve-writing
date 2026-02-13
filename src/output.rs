use anyhow::{Context, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Type text at the cursor position.
///
/// - Linux: uses `wtype` (Wayland)
/// - macOS: uses `osascript` with AppleScript `keystroke`
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "macos")]
pub async fn type_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    // Escape backslashes and double quotes for AppleScript string literal
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "System Events" to keystroke "{}""#,
        escaped
    );

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .await
        .context("Failed to type text via osascript (check Accessibility permissions)")?;

    Ok(())
}

/// Copy text to the system clipboard.
///
/// - Linux: uses `wl-copy`
/// - macOS: uses `pbcopy`
#[cfg(target_os = "linux")]
pub async fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to run wl-copy (is wl-clipboard installed?)")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).await?;
    }

    child.wait().await.context("wl-copy failed")?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to run pbcopy")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).await?;
    }

    child.wait().await.context("pbcopy failed")?;
    Ok(())
}

/// Get selected text.
///
/// - Linux: reads the Wayland primary selection via `wl-paste --primary`
/// - macOS: simulates Cmd+C to copy highlighted text, then reads via `pbpaste`
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "macos")]
pub async fn get_primary_selection() -> Result<String> {
    // Simulate Cmd+C to copy the currently highlighted text to the clipboard
    Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "c" using command down"#)
        .status()
        .await
        .context("Failed to simulate Cmd+C via osascript (check Accessibility permissions)")?;

    // Brief delay to let the clipboard populate
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let output = Command::new("pbpaste")
        .output()
        .await
        .context("Failed to get clipboard contents (is pbpaste available?)")?;

    if !output.status.success() {
        anyhow::bail!("pbpaste failed: {:?}", output.status);
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

/// Clear the current terminal line by sending Ctrl+U.
///
/// - Linux: uses `wtype` to simulate Ctrl+U
/// - macOS: uses `osascript` to simulate Ctrl+U
#[cfg(target_os = "linux")]
pub async fn clear_line() -> Result<()> {
    Command::new("wtype")
        .args(["-M", "ctrl", "-k", "u", "-m", "ctrl"])
        .status()
        .await
        .context("Failed to clear line (is wtype installed?)")?;

    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn clear_line() -> Result<()> {
    Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "u" using control down"#)
        .status()
        .await
        .context("Failed to clear line via osascript (check Accessibility permissions)")?;

    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}
