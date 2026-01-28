# improve-writing

A hotkey-triggered text improvement tool for Linux/Wayland. Select text, press a hotkey, and get an improved version typed back via Ollama.

## How it works

1. Press the hotkey (default: `F8`)
2. The tool grabs highlighted text via `wl-paste --primary`
3. Sends it to Ollama with a prompt to improve clarity, grammar, and style
4. Types the improved text back via `wtype`

## Requirements

- Linux with Wayland
- `wl-clipboard` (provides `wl-paste`)
- `wtype` (for typing text)
- [Ollama](https://ollama.ai/) running with a model pulled
- User must be in the `input` group (or run as root) for keyboard access

### Install dependencies (Fedora)

```bash
sudo dnf install wl-clipboard wtype
sudo usermod -aG input $USER
# Log out and back in for group change to take effect
```

### Install dependencies (Ubuntu/Debian)

```bash
sudo apt install wl-clipboard wtype
sudo usermod -aG input $USER
# Log out and back in for group change to take effect
```

### Install Ollama model

```bash
ollama pull qwen3:1.7b
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
# Default settings (F8, qwen3:1.7b)
./target/release/improve-writing

# Custom hotkey
./target/release/improve-writing --key F10
./target/release/improve-writing --key Ctrl+F9
./target/release/improve-writing --key Ctrl+Alt+F1

# Different model
./target/release/improve-writing --ollama-model qwen2.5:1.5b

# Verbose logging
./target/release/improve-writing --verbose

# Show original text followed by improved text
./target/release/improve-writing --show-original
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--key` | `F8` | Hotkey to trigger (F1-F12, ScrollLock, Pause, Insert with optional Shift/Ctrl/Alt) |
| `--ollama-host` | `http://localhost` | Ollama host URL |
| `--ollama-port` | `11434` | Ollama port |
| `--ollama-model` | `qwen3:1.7b` | Ollama model to use |
| `--verbose` | off | Enable debug logging |
| `--show-original` | off | Output original text followed by improved text |

## License

MIT
