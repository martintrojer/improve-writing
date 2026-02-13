# CLAUDE.md

## Project Overview

improve-writing is a Rust CLI tool that listens for global hotkeys, grabs selected text, sends it to Ollama for improvement or shell command generation, and types the result back. Supports Linux (Wayland) and macOS.

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
├── event_loop.rs  # Main loop, hotkey detection, mode dispatch (improve/command)
├── ollama.rs      # Ollama API integration (TextImprover: improve text, generate commands)
├── output.rs      # Clipboard, typing, and line clearing (wl-paste/wtype on Linux, pbpaste/pbcopy/osascript on macOS)
```

## Key Dependencies

- `hotkey-listener` - Cross-platform global hotkey listening
- `ollama-rs` - Ollama API client
- `tokio` - Async runtime
- `clap` - CLI argument parsing

### Platform-specific

- **Linux (Wayland):** `wl-paste`/`wl-copy` (wl-clipboard) and `wtype` for clipboard/typing
- **macOS:** `pbpaste`/`pbcopy` (built-in) and `osascript` for typing via AppleScript

## Architecture Notes

- Keyboard listener runs in a separate thread, communicates via mpsc channel
- Modifier keys (Shift/Ctrl/Alt) are tracked to support hotkey combinations
- Ollama client uses custom reqwest settings: 120s timeout, disabled connection pooling, 3 retries
- Shared `send_chat` method handles retries for both text improvement and command generation prompts
- Platform-specific output via `#[cfg(target_os = "...")]` in `output.rs`

## Testing Manually

### Linux (Wayland)

1. Ensure Ollama is running: `ollama serve`
2. Pull model: `ollama pull qwen3:1.7b` (also a particularly good choice: `qwen3:4b-instruct`)
3. Run: `cargo run -- --verbose`
4. Select text in any application
5. Press F8 for improved text, or Shift+F8 for original + improved
6. Improved text is typed at cursor position
7. In a terminal, type a command description, select it, press F7
8. The line is cleared and the generated shell command is typed

### macOS

1. Ensure Ollama is running: `ollama serve`
2. Pull model: `ollama pull qwen3:1.7b` (also a particularly good choice: `qwen3:4b-instruct`)
3. Grant Accessibility permissions to your terminal (System Settings > Privacy & Security > Accessibility)
4. Run: `cargo run -- --verbose`
5. Select text in any application
6. Press F8 for improved text, or Shift+F8 for original + improved
7. Improved text is typed at cursor position
8. In a terminal, type a command description, select it, press F7
9. The line is cleared and the generated shell command is typed
