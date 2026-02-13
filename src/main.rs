mod event_loop;
mod ollama;
mod output;

use anyhow::Result;
use clap::Parser;
use hotkey_listener::{HotkeyListenerBuilder, parse_hotkey};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::Release};

#[derive(Parser, Debug)]
#[command(name = "improve-writing")]
#[command(about = "Hotkey-triggered text improvement via Ollama")]
struct Args {
    /// Hotkey to trigger text improvement (e.g., F9, Shift+F9, Ctrl+Alt+F1)
    #[arg(long, default_value = "F8")]
    key: String,

    /// Hotkey to output original + improved text (default: Shift+<key>)
    #[arg(long)]
    show_original_key: Option<String>,

    /// Hotkey to generate a shell command from a description
    #[arg(long, default_value = "F7")]
    cmd_key: String,

    /// Ollama host URL
    #[arg(long, default_value = "http://localhost")]
    ollama_host: String,

    /// Ollama port
    #[arg(long, default_value_t = 11434)]
    ollama_port: u16,

    /// Ollama model to use
    #[arg(long, default_value = "qwen3:1.7b")]
    ollama_model: String,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Parse hotkeys
    let hotkey = parse_hotkey(&args.key)?;
    let show_original_hotkey = match &args.show_original_key {
        Some(key) => parse_hotkey(key)?,
        None => hotkey.with_shift(),
    };
    log::info!("Hotkey: {}", hotkey);
    log::info!("Show-original hotkey: {}", show_original_hotkey);

    let cmd_hotkey = parse_hotkey(&args.cmd_key)?;
    log::info!("Shell command hotkey: {}", cmd_hotkey);

    #[cfg(target_os = "macos")]
    log::info!("Note: You may need to grant Accessibility permissions for osascript to type text.");

    // Build and start the hotkey listener
    // Index 0 = main hotkey (improve only)
    // Index 1 = show original hotkey (improve + show original)
    // Index 2 = shell command hotkey (generate command)
    let handle = HotkeyListenerBuilder::new()
        .add_hotkey(hotkey)
        .add_hotkey(show_original_hotkey)
        .add_hotkey(cmd_hotkey)
        .build()?
        .start()?;

    // Create text improver
    let improver =
        ollama::TextImprover::new(&args.ollama_host, args.ollama_port, &args.ollama_model);
    log::debug!(
        "Using Ollama at {}:{} with model {}",
        args.ollama_host,
        args.ollama_port,
        args.ollama_model
    );

    // Setup Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        log::info!("Received Ctrl+C, shutting down...");
        r.store(false, Release);
    })?;

    // Run the event loop
    event_loop::run_event_loop(handle, improver, running).await?;

    log::info!("Goodbye!");
    Ok(())
}
