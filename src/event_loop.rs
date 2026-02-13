use anyhow::Result;
use hotkey_listener::{HotkeyEvent, HotkeyListenerHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::Acquire};
use std::time::Duration;

use crate::ollama::TextImprover;
use crate::output::{clear_line, copy_to_clipboard, get_primary_selection, type_text};

enum Mode {
    Improve,
    ImproveShowOriginal,
    ShellCommand,
}

/// Check for the REDO keyword (whole word, all-caps). Returns the cleaned text
/// with REDO stripped and whether refinement was requested.
fn extract_refine(text: &str) -> (String, bool) {
    let has_redo = text.split_whitespace().any(|w| w == "REDO");
    if has_redo {
        let cleaned = text
            .split_whitespace()
            .filter(|w| *w != "REDO")
            .collect::<Vec<_>>()
            .join(" ");
        (cleaned, true)
    } else {
        (text.to_string(), false)
    }
}

pub async fn run_event_loop(
    handle: HotkeyListenerHandle,
    mut improver: TextImprover,
    running: Arc<AtomicBool>,
) -> Result<()> {
    log::info!("Listening for hotkey... Press Ctrl+C to exit.");

    while running.load(Acquire) {
        // Check for hotkey events
        match handle.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                // Only handle press events, not releases
                let mode = match event {
                    HotkeyEvent::Pressed(0) => Mode::Improve,
                    HotkeyEvent::Pressed(1) => Mode::ImproveShowOriginal,
                    HotkeyEvent::Pressed(2) => Mode::ShellCommand,
                    _ => continue,
                };

                log::info!("Hotkey pressed - getting selection...");

                // Get highlighted text
                match get_primary_selection().await {
                    Ok(text) => {
                        let text = text.trim();
                        if text.is_empty() {
                            log::warn!("No text selected");
                            continue;
                        }

                        log::debug!("Selected text: {:?}", text);

                        // Copy original text to clipboard as backup
                        if let Err(e) = copy_to_clipboard(text).await {
                            log::warn!("Failed to copy original to clipboard: {}", e);
                        } else {
                            log::debug!("Original text copied to clipboard");
                        }

                        match mode {
                            Mode::Improve | Mode::ImproveShowOriginal => {
                                let show_original = matches!(mode, Mode::ImproveShowOriginal);
                                let (input, refine) = if show_original {
                                    (text.to_string(), false)
                                } else {
                                    extract_refine(text)
                                };

                                match improver.improve(&input, refine).await {
                                    Ok(improved) => {
                                        if improved.is_empty() {
                                            log::warn!("Ollama returned empty response");
                                            continue;
                                        }

                                        log::debug!("Improved text: {:?}", improved);

                                        let improved_clean = improved.replace('\n', "  ");
                                        let output = if show_original {
                                            let text_clean = text.replace('\n', "  ");
                                            format!("{} | {}", text_clean, improved_clean)
                                        } else {
                                            improved_clean
                                        };

                                        if let Err(e) = type_text(&output).await {
                                            log::error!("Failed to type text: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to improve text: {}", e);
                                    }
                                }
                            }
                            Mode::ShellCommand => {
                                let (input, refine) = extract_refine(text);
                                match improver.generate_command(&input, refine).await {
                                    Ok(command) => {
                                        if command.is_empty() {
                                            log::warn!("Ollama returned empty response");
                                            continue;
                                        }

                                        log::debug!("Generated command: {:?}", command);

                                        if let Err(e) = clear_line().await {
                                            log::error!("Failed to clear line: {}", e);
                                        }

                                        if let Err(e) = type_text(&command).await {
                                            log::error!("Failed to type command: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to generate command: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get selection: {}", e);
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No event, continue loop
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log::debug!("Keyboard listener disconnected");
                break;
            }
        }
    }

    Ok(())
}
