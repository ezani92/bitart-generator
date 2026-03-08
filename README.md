# BitArt Generator

A terminal-based pixel art generator built in Rust. Generate pixel art from text prompts using OpenAI image generation (DALL-E & GPT Image models). View art directly in your terminal with high-resolution half-block rendering and export to PNG or animated GIF.

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **AI Generation** — Uses OpenAI image API to generate pixel art from text prompts
- **5 Models** — DALL-E 2, DALL-E 3, GPT Image Mini, GPT Image 1, GPT Image 1.5
- **PNG & GIF Export** — Save as PNG or animated GIF (3 frames at 3fps) at native API resolution (1024x1024)
- **High-Res Terminal Preview** — Half-block (`▀`) rendering for maximum terminal resolution
- **GIF Animation** — 3-frame animation using iterative edits API — each frame is contextually aware of the previous ones for consistent results
- **Mode Toggle** — Switch between PNG and GIF mode with `Shift+Tab`
- **Config Menu** — Change model/API key or save folder anytime with `Ctrl+C`
- **Default Save Folder** — Configure a default output directory with timestamped filenames
- **Transparent Export** — Near-white pixels automatically become transparent in exported PNG
- **CLI Mode** — Generate art directly from command line with `-p`, `-o`, and `-g` flags
- **Loading Quotes** — Inspirational quotes rotate during generation
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

1. **Select a model** — 5 options from budget to best quality
2. **Enter your OpenAI API key** — Get one at [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
3. **Set output folder** — Default save location for exported files (defaults to `~/Downloads/bitart/`)

Your config is saved to `~/.bitart/config.json` and persists across sessions.

### Available Models

| Model | Price | Notes |
|-------|-------|-------|
| DALL-E 2 | $0.02/image | Cheapest |
| DALL-E 3 | $0.04/image | Better quality |
| GPT Image Mini | $0.02/image | Fast |
| GPT Image 1 | $0.04/image | Great quality |
| GPT Image 1.5 | $0.04/image | Best quality |

## Usage

### Interactive TUI

```bash
bitart
```

This opens the TUI. Type a prompt and press Enter to generate pixel art. Use `Shift+Tab` to toggle between PNG and GIF mode.

### CLI Mode

Generate art directly without the TUI:

```bash
# Generate PNG
bitart -p "oak tree"

# Generate with custom output path
bitart -p "sunset mountain" -o ~/Downloads/sunset.png

# Generate animated GIF (3 frames at 3fps)
bitart -p "dancing cat" -g

# GIF with custom path
bitart -p "campfire" -g -o fire.gif

# Show help
bitart -h
```

### Keybindings (TUI)

| Key | Action |
|-----|--------|
| `Shift+Tab` | Toggle PNG/GIF mode |
| `Enter` | Generate art from prompt / New prompt |
| `Ctrl+N` | New prompt |
| `Ctrl+S` | Save to configured output folder |
| `Ctrl+R` | Regenerate same prompt |
| `Ctrl+C` | Open settings menu |
| `Ctrl+Q` | Quit |
| `Esc` | Quit |

### Settings Menu (`Ctrl+C`)

| Option | Description |
|--------|-------------|
| Update Model & API Key | Change AI model and OpenAI API key |
| Update Save Folder | Set default output directory |

## How It Works

1. **Enter a prompt** — e.g. "skeleton knight", "neon city skyline", "pixel art cat"
2. **Generation** — Sends prompt to OpenAI image API, receives image at native resolution
3. **Preview** — Art is displayed in your terminal using half-block `▀` characters with fg/bg colors for 2x vertical resolution
4. **Export** — Press `Ctrl+S` to save as PNG (with transparent background) or animated GIF

### GIF Animation

In GIF mode, 3 frames are generated using the OpenAI edits API:
- **Frame 1**: Base image generated from your prompt
- **Frame 2**: Edits API receives frame 1 as reference — creates a subtle animation change
- **Frame 3**: Edits API receives frames 1 & 2 — completes the animation loop

This ensures each frame is contextually aware of the previous frames for consistent, drift-free animation.

## Project Structure

```
src/
├── main.rs              # Entry point + CLI mode
├── config.rs            # API key, model, output dir config (~/.bitart/config.json)
├── generator/
│   ├── mod.rs           # Async generation orchestrator (single + multi-frame)
│   └── dalle.rs         # OpenAI image API (generations + edits)
├── tui/
│   └── mod.rs           # Ratatui TUI (setup, config menu, input, canvas, status)
└── exporter/
    └── mod.rs           # PNG (transparent) & GIF export
```

## License

MIT
