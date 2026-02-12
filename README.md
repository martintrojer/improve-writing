# improve-writing

A hotkey-triggered text improvement tool for Linux (Wayland) and macOS. Select text, press a hotkey, and get an improved version typed back via Ollama.

## How it works

1. Press the hotkey (default: `F8`) to get improved text
2. Press `Shift+F8` to get original + improved text
3. The tool grabs highlighted text (via `wl-paste --primary` on Linux, or simulated `Cmd+C` on macOS)
4. Sends it to Ollama with a prompt to improve clarity, grammar, and style
5. Types the improved text back (via `wtype` on Linux, or `osascript` on macOS)

## Requirements

- [Ollama](https://ollama.ai/) running with a model pulled

### Linux (Wayland)

- `wl-clipboard` (provides `wl-paste`/`wl-copy`)
- `wtype` (for typing text)

#### Install dependencies (Fedora)

```bash
sudo dnf install wl-clipboard wtype
```

#### Install dependencies (Ubuntu/Debian)

```bash
sudo apt install wl-clipboard wtype
```

### macOS

- Grant Accessibility permissions to your terminal (System Settings > Privacy & Security > Accessibility)
- `pbcopy`/`pbpaste` (built-in) and `osascript` (built-in) are used automatically

### Install Ollama model

```bash
ollama pull qwen3:1.7b
# Also a particularly good choice:
ollama pull qwen3:4b-instruct
```

## Building

```bash
cargo build --release
```

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Default settings (F8 for improved, Shift+F8 for original+improved)
./target/release/improve-writing

# Custom hotkey (Shift+<key> automatically set for show-original)
./target/release/improve-writing --key F10

# Custom show-original hotkey
./target/release/improve-writing --key F8 --show-original-key Ctrl+F8

# Different model
./target/release/improve-writing --ollama-model qwen2.5:1.5b

# Verbose logging
./target/release/improve-writing --verbose
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--key` | `F8` | Hotkey for improved text only |
| `--show-original-key` | `Shift+<key>` | Hotkey for original + improved text |
| `--ollama-host` | `http://localhost` | Ollama host URL |
| `--ollama-port` | `11434` | Ollama port |
| `--ollama-model` | `qwen3:1.7b` | Ollama model to use (also a particularly good choice: `qwen3:4b-instruct`) |
| `--verbose` | off | Enable debug logging |

## License

MIT
