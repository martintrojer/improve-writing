# CLAUDE.md

## Project Overview

improve-writing is a Rust CLI tool that listens for a global hotkey, grabs highlighted text via Wayland primary selection, sends it to Ollama for improvement, and types the result back.

## Build & Run

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run with defaults
cargo run -- --verbose   # Run with debug logging
```

## Before Committing

Always run formatting and linting before committing:

```bash
cargo fmt
cargo clippy
```

## Project Structure

```
src/
├── main.rs        # Entry point, CLI args, initialization
├── input.rs       # Hotkey parsing, keyboard device discovery (evdev)
├── event_loop.rs  # Main loop, modifier tracking, hotkey detection
├── ollama.rs      # Ollama API integration (TextImprover)
├── output.rs      # wl-paste (get selection) and wtype (type text)
```

## Key Dependencies

- `evdev` - Linux keyboard input via /dev/input
- `ollama-rs` - Ollama API client
- `tokio` - Async runtime
- `clap` - CLI argument parsing

## Architecture Notes

- Keyboard listener runs in a separate thread, communicates via mpsc channel
- Modifier keys (Shift/Ctrl/Alt) are tracked to support hotkey combinations
- Ollama client uses custom reqwest settings: 120s timeout, disabled connection pooling, 3 retries
- Linux-only (Wayland) - uses wl-paste/wtype for clipboard/typing

## Testing Manually

1. Ensure Ollama is running: `ollama serve`
2. Pull model: `ollama pull qwen3:1.7b`
3. Run: `cargo run -- --verbose`
4. Select text in any application
5. Press F8
6. Improved text is typed at cursor position
