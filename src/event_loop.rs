use anyhow::Result;
use hotkey_listener::{HotkeyEvent, HotkeyListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::ollama::TextImprover;
use crate::output::{copy_to_clipboard, get_primary_selection, type_text};

pub async fn run_event_loop(
    listener: HotkeyListener,
    improver: TextImprover,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let rx = listener.start(running.clone())?;

    log::info!("Listening for hotkey... Press Ctrl+C to exit.");

    while running.load(Ordering::Relaxed) {
        // Check for hotkey events
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                // Index 0 = main hotkey (improve only)
                // Index 1 = show original hotkey (improve + show original)
                let show_original = matches!(event, HotkeyEvent::Pressed(1));

                // Only handle press events, not releases
                if !matches!(event, HotkeyEvent::Pressed(_)) {
                    continue;
                }

                log::info!("Hotkey pressed - getting selection and improving...");

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

                        // Improve text via Ollama
                        match improver.improve(text).await {
                            Ok(improved) => {
                                if improved.is_empty() {
                                    log::warn!("Ollama returned empty response");
                                    continue;
                                }

                                log::debug!("Improved text: {:?}", improved);

                                // Build output text (strip newlines to avoid triggering send in chat tools)
                                let improved_clean = improved.replace('\n', "  ");
                                let output = if show_original {
                                    let text_clean = text.replace('\n', "  ");
                                    format!("{} | {}", text_clean, improved_clean)
                                } else {
                                    improved_clean
                                };

                                // Type the text
                                if let Err(e) = type_text(&output).await {
                                    log::error!("Failed to type text: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to improve text: {}", e);
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
