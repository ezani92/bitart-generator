# BitArt Generator

A terminal-based pixel art generator built in Rust. Generate 64x64 pixel art from text prompts using DALL-E image generation. View art directly in your terminal and export to PNG.

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **AI Generation** — Uses OpenAI DALL-E API to generate pixel art from text prompts
- **Model Selection** — Choose between DALL-E 2 ($0.02/image) or DALL-E 3 ($0.04/image)
- **Setup Wizard** — First-launch setup for model selection and API key entry
- **Config Management** — Change model or API key anytime with `c` keybind
- **Terminal Preview** — Full-color pixel art rendered with Unicode block characters
- **PNG Export** — Save art as 512x512 PNG (8x upscaled from 64x64)
- **Interactive TUI** — Built with Ratatui for a smooth terminal experience

## Installation

### npm (Easiest)

```bash
npm install -g bitart-generator
```

### Homebrew (macOS/Linux)

```bash
brew tap ezani92/bitart
brew install bitart
```

### Cargo (From Source)

Requires [Rust](https://rustup.rs/) 1.70+:

```bash
cargo install bitart-generator
```

### From Releases

Download the latest binary from [Releases](https://github.com/ezani92/bitart-generator/releases).

## Setup

On first launch, BitArt will prompt you to:

1. **Select a model** — DALL-E 2 (cheapest) or DALL-E 3 (better quality)
2. **Enter your OpenAI API key** — Get one at [platform.openai.com/api-keys](https://platform.openai.com/api-keys)

Your config is saved to `~/.bitart/config.json` and persists across sessions.

## Usage

```bash
bitart
```

This opens the TUI. Type a prompt and press Enter to generate pixel art.

### Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Generate art from prompt / New prompt |
| `s` | Save to `output.png` |
| `r` | Regenerate same prompt |
| `c` | Open config (change model/API key) |
| `q` | Quit |
| `Esc` | Quit |

## How It Works

1. **Enter a prompt** — e.g. "neon city skyline", "sunset mountains", "pixel art cat"
2. **Generation** — Sends prompt to DALL-E API, downloads the image, and downscales to 64x64 pixels
3. **Preview** — Art is displayed in your terminal using colored `██` Unicode blocks
4. **Export** — Press `s` to save as a 512x512 PNG to `./output.png`

## Project Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # API key & model config (~/.bitart/config.json)
├── generator/
│   ├── mod.rs           # Async generation orchestrator
│   └── dalle.rs         # DALL-E API integration
├── tui/
│   └── mod.rs           # Ratatui TUI (setup, input, canvas, status)
└── exporter/
    └── mod.rs           # PNG export (image crate)
```

## License

MIT
