# Pixelens

A Linux-native visual search and OCR utility.

## Features

- **Screen Capture**: Select any region of your screen using grim/slurp
- **OCR**: Extract text from captured images using Tesseract
- **AI Integration**: Ask AI about captured content (OpenAI-compatible APIs)
- **Actions**: Copy text, web search, reverse image search, translate

## Architecture

```
pixelens/
├── crates/
│   ├── pixelens-common/     # Shared types and errors
│   ├── pixelens-config/     # Configuration management
│   ├── pixelens-capture/    # Screen capture (grim/slurp)
│   ├── pixelens-ocr/        # OCR (Tesseract)
│   ├── pixelens-actions/    # Action handlers
│   ├── pixelens-cli/        # CLI binary
│   └── pixelensd/           # Daemon binary
└── docs/
```

## Requirements

- Rust 1.77.2+
- grim (Wayland screenshot tool)
- slurp (Wayland region selector)
- tesseract-ocr

## Installation

```bash
cargo install --path crates/pixelens-cli
```

## Usage

```bash
# Capture a region
pixelens capture

# Perform OCR on an image
pixelens ocr --image screenshot.png

# Ask AI about an image
pixelens ai --prompt "What is this?" --image screenshot.png

# Execute an action
pixelens action --name search --text "rust programming"

# Check required tools
pixelens check

# Show configuration
pixelens config
```

## Configuration

Configuration is stored at `~/.config/pixelens/config.json`:

```json
{
  "api_endpoint": "https://api.openai.com/v1",
  "api_key": "sk-...",
  "model": "gpt-4o",
  "ocr_language": "eng",
  "hotkey": "Ctrl+Shift+C"
}
```

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT
