use anyhow::Result;
use evdev::{Device, Key};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::input::{Hotkey, Modifiers};
use crate::ollama::TextImprover;
use crate::output::{get_primary_selection, type_text};

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    Triggered,
}

fn start_keyboard_listener(
    mut keyboards: Vec<Device>,
    hotkey: Hotkey,
    running: Arc<AtomicBool>,
    tx: Sender<HotkeyEvent>,
) -> Result<()> {
    // Set non-blocking mode on all devices
    for device in &keyboards {
        let fd = device.as_raw_fd();
        let flags = fcntl(fd, FcntlArg::F_GETFL)?;
        let flags = OFlag::from_bits_truncate(flags);
        fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;
    }

    thread::spawn(move || {
        // Track current modifier state
        let mut current_mods = Modifiers::default();

        while running.load(Ordering::Relaxed) {
            for device in keyboards.iter_mut() {
                if let Ok(events) = device.fetch_events() {
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
                                    current_mods.ctrl = pressed || (!released && current_mods.ctrl);
                                    if released {
                                        current_mods.ctrl = false;
                                    }
                                }
                                Key::KEY_LEFTALT | Key::KEY_RIGHTALT => {
                                    current_mods.alt = pressed || (!released && current_mods.alt);
                                    if released {
                                        current_mods.alt = false;
                                    }
                                }
                                _ => {}
                            }

                            // Check if hotkey is triggered
                            if key == hotkey.key && pressed {
                                // Check if required modifiers match
                                let mods_match = current_mods.shift == hotkey.modifiers.shift
                                    && current_mods.ctrl == hotkey.modifiers.ctrl
                                    && current_mods.alt == hotkey.modifiers.alt;

                                if mods_match {
                                    let _ = tx.send(HotkeyEvent::Triggered);
                                }
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    Ok(())
}

pub async fn run_event_loop(
    keyboards: Vec<Device>,
    hotkey: Hotkey,
    improver: TextImprover,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let (tx, rx): (Sender<HotkeyEvent>, Receiver<HotkeyEvent>) = mpsc::channel();

    start_keyboard_listener(keyboards, hotkey, running.clone(), tx)?;

    log::info!("Listening for hotkey... Press Ctrl+C to exit.");

    while running.load(Ordering::Relaxed) {
        // Check for hotkey events
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(HotkeyEvent::Triggered) => {
                log::info!("Hotkey pressed - getting selection and improving...");

                // Get highlighted text
                match get_primary_selection().await {
                    Ok(text) => {
                        let text = text.trim();
                        if text.is_empty() {
                            log::warn!("No text selected");
                            continue;
                        }

                        log::info!("Selected text: {:?}", text);

                        // Improve text via Ollama
                        match improver.improve(text).await {
                            Ok(improved) => {
                                if improved.is_empty() {
                                    log::warn!("Ollama returned empty response");
                                    continue;
                                }

                                log::info!("Improved text: {:?}", improved);

                                // Type the improved text
                                if let Err(e) = type_text(&improved).await {
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
                log::info!("Keyboard listener disconnected");
                break;
            }
        }
    }

    Ok(())
}
