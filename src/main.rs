mod event_loop;
mod input;
mod ollama;
mod output;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser, Debug)]
#[command(name = "improve-writing")]
#[command(about = "Hotkey-triggered text improvement via Ollama")]
struct Args {
    /// Hotkey to trigger text improvement (e.g., F9, Shift+F9, Ctrl+Alt+F1)
    #[arg(long, default_value = "Shift+F10")]
    key: String,

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

    /// Output original text followed by improved text
    #[arg(long)]
    show_original: bool,
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

    // Parse hotkey
    let hotkey = input::parse_hotkey(&args.key)?;
    log::info!("Using hotkey: {}", args.key);

    // Find keyboards
    let keyboards = input::find_keyboards()?;
    log::info!("Found {} keyboard(s)", keyboards.len());

    // Create text improver
    let improver =
        ollama::TextImprover::new(&args.ollama_host, args.ollama_port, &args.ollama_model);
    log::info!(
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
        r.store(false, Ordering::Relaxed);
    })?;

    // Run the event loop
    event_loop::run_event_loop(keyboards, hotkey, improver, running, args.show_original).await?;

    log::info!("Goodbye!");
    Ok(())
}
