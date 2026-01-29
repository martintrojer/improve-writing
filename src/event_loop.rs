use anyhow::Result;
use evdev::{Device, Key};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::input::{Hotkey, Modifiers, find_keyboards};
use crate::ollama::TextImprover;
use crate::output::{copy_to_clipboard, get_primary_selection, type_text};

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    Improve,
    ImproveShowOriginal,
}

/// Set non-blocking mode on keyboard devices
fn set_nonblocking(keyboards: &[Device]) {
    for device in keyboards {
        let fd = device.as_raw_fd();
        if let Ok(flags) = fcntl(fd, FcntlArg::F_GETFL) {
            let flags = OFlag::from_bits_truncate(flags);
            let _ = fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK));
        }
    }
}

fn start_keyboard_listener(
    keyboards: Vec<Device>,
    hotkey: Hotkey,
    show_original_hotkey: Option<Hotkey>,
    running: Arc<AtomicBool>,
    tx: Sender<HotkeyEvent>,
) -> Result<()> {
    set_nonblocking(&keyboards);

    thread::spawn(move || {
        let mut keyboards = keyboards;
        let mut current_mods = Modifiers::default();
        let mut last_rescan = Instant::now();
        let mut had_error = false;

        // Minimum interval between keyboard rescans
        const RESCAN_INTERVAL: Duration = Duration::from_secs(10);

        while running.load(Ordering::Relaxed) {
            // Check if we need to rescan keyboards (after error and interval passed)
            if had_error && last_rescan.elapsed() >= RESCAN_INTERVAL {
                log::info!("Keyboard error detected, rescanning devices...");
                match find_keyboards() {
                    Ok(new_keyboards) => {
                        log::info!(
                            "Keyboards reconnected: found {} device(s)",
                            new_keyboards.len()
                        );
                        set_nonblocking(&new_keyboards);
                        keyboards = new_keyboards;
                        current_mods = Modifiers::default(); // Reset modifier state
                        had_error = false;
                    }
                    Err(e) => {
                        log::warn!("Failed to rescan keyboards: {}", e);
                    }
                }
                last_rescan = Instant::now();
            }

            let mut any_error = false;

            for device in keyboards.iter_mut() {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let evdev::InputEventKind::Key(key) = event.kind() {
                                let pressed = event.value() == 1;
                                let released = event.value() == 0;

                                // Track modifier state
                                match key {
                                    Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => {
                                        current_mods.shift =
                                            pressed || (!released && current_mods.shift);
                                        if released {
                                            current_mods.shift = false;
                                        }
                                    }
                                    Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => {
                                        current_mods.ctrl =
                                            pressed || (!released && current_mods.ctrl);
                                        if released {
                                            current_mods.ctrl = false;
                                        }
                                    }
                                    Key::KEY_LEFTALT | Key::KEY_RIGHTALT => {
                                        current_mods.alt =
                                            pressed || (!released && current_mods.alt);
                                        if released {
                                            current_mods.alt = false;
                                        }
                                    }
                                    _ => {}
                                }

                                // Check show_original hotkey first (more specific)
                                if let Some(ref so_hotkey) = show_original_hotkey
                                    && key == so_hotkey.key
                                    && pressed
                                {
                                    let mods_match = current_mods.shift
                                        == so_hotkey.modifiers.shift
                                        && current_mods.ctrl == so_hotkey.modifiers.ctrl
                                        && current_mods.alt == so_hotkey.modifiers.alt;

                                    if mods_match {
                                        let _ = tx.send(HotkeyEvent::ImproveShowOriginal);
                                        continue;
                                    }
                                }

                                // Check normal hotkey
                                if key == hotkey.key && pressed {
                                    let mods_match = current_mods.shift == hotkey.modifiers.shift
                                        && current_mods.ctrl == hotkey.modifiers.ctrl
                                        && current_mods.alt == hotkey.modifiers.alt;

                                    if mods_match {
                                        let _ = tx.send(HotkeyEvent::Improve);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // EAGAIN/EWOULDBLOCK is expected for non-blocking reads
                        if e.raw_os_error() != Some(libc::EAGAIN)
                            && e.raw_os_error() != Some(libc::EWOULDBLOCK)
                        {
                            log::debug!("Keyboard read error: {}", e);
                            any_error = true;
                        }
                    }
                }
            }

            if any_error {
                had_error = true;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    Ok(())
}

pub async fn run_event_loop(
    keyboards: Vec<Device>,
    hotkey: Hotkey,
    show_original_hotkey: Option<Hotkey>,
    improver: TextImprover,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let (tx, rx): (Sender<HotkeyEvent>, Receiver<HotkeyEvent>) = mpsc::channel();

    start_keyboard_listener(keyboards, hotkey, show_original_hotkey, running.clone(), tx)?;

    log::info!("Listening for hotkey... Press Ctrl+C to exit.");

    while running.load(Ordering::Relaxed) {
        // Check for hotkey events
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                let show_original = matches!(event, HotkeyEvent::ImproveShowOriginal);
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
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No event, continue loop
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::debug!("Keyboard listener disconnected");
                break;
            }
        }
    }

    Ok(())
}
