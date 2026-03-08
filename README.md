# BitArt Generator

A terminal-based pixel art generator built in Rust. Generate 64x64 pixel art from text prompts using Claude AI or math-based fallback patterns. View art directly in your terminal and export to PNG.

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **AI Generation** — Uses Claude CLI to generate pixel art from text prompts
- **Math Fallback** — Hash-seeded sine wave + XOR patterns when Claude is unavailable
- **Keyword Palettes** — Automatic color palette selection based on prompt keywords (neon, warm, cool, nature)
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

Download the latest binary from [Releases](https://github.com/ezani92/bitart-generator/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/ezani92/bitart-generator/releases/latest/download/bitart-v0.1.0-macos-arm64.tar.gz | tar xz
sudo mv bitart /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/ezani92/bitart-generator/releases/latest/download/bitart-v0.1.0-macos-x86_64.tar.gz | tar xz
sudo mv bitart /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/ezani92/bitart-generator/releases/latest/download/bitart-v0.1.0-linux-x86_64.tar.gz | tar xz
sudo mv bitart /usr/local/bin/
```

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
| `r` | Regenerate with new seed |
| `q` | Quit |
| `Esc` | Quit |

### AI Mode (Optional)

If you have [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed, BitArt will use it to generate pixel art from your prompts. If Claude isn't available, it falls back to math-based pattern generation automatically.

```bash
# Install Claude Code CLI (optional, for AI generation)
npm install -g @anthropic-ai/claude-code
```

## How It Works

1. **Enter a prompt** — e.g. "neon city skyline", "sunset mountains", "forest lake"
2. **Generation** — Tries Claude AI first; if unavailable, generates patterns using sine waves and XOR bitwise operations seeded from your prompt
3. **Preview** — Art is displayed in your terminal using colored `██` Unicode blocks
4. **Export** — Press `s` to save as a 512x512 PNG to `./output.png`

### Keyword Palettes

The math fallback selects colors based on keywords in your prompt:

| Keywords | Palette |
|----------|---------|
| neon, cyber, glow | Magenta, cyan, yellow, green |
| warm, fire, sunset, lava | Reds, oranges, golds |
| cool, ice, ocean, water | Blues, teals, light blues |
| nature, forest, tree, grass | Greens, browns, olives |
| *(default)* | Rainbow mix |

## Project Structure

```
src/
├── main.rs              # Entry point
├── generator/
│   ├── mod.rs           # Orchestrator (AI vs fallback)
│   ├── claude.rs        # Claude CLI integration
│   ├── fallback.rs      # Math-based pattern generation
│   └── palette.rs       # Keyword → color palettes
├── tui/
│   └── mod.rs           # Ratatui TUI (input, canvas, status)
└── exporter/
    └── mod.rs           # PNG export (image crate)
```

## License

MIT
