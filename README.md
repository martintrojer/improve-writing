# improve-writing

A hotkey-triggered text improvement tool for Linux/Wayland. Select text, press a hotkey, and get an improved version typed back via Ollama.

## How it works

1. Press the hotkey (default: `F8`) to get improved text
2. Press `Shift+F8` to get original + improved text
3. The tool grabs highlighted text via `wl-paste --primary`
4. Sends it to Ollama with a prompt to improve clarity, grammar, and style
5. Types the improved text back via `wtype`

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
| `--ollama-model` | `qwen3:1.7b` | Ollama model to use |
| `--verbose` | off | Enable debug logging |

## License

MIT
