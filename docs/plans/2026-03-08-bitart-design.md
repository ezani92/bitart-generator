# BitArt — Pixel Art Generator TUI

## Overview

A Rust TUI app that generates 64x64 pixel art from text prompts. Uses Claude CLI for AI generation with a math-based fallback. Displays art in-terminal using colored Unicode blocks and exports to PNG.

## Architecture

Three modules under `src/`:

### generator/ — Pixel Art Creation

- `mod.rs`: Orchestrator — tries Claude, falls back to math on failure
- `claude.rs`: Shells out to `claude -p "..."`, parses stdout as JSON `Vec<Vec<String>>` (64x64 hex colors), 60s timeout
- `fallback.rs`: Hash prompt string → seed, compute `f(x, y, seed)` using sine waves + XOR bitwise patterns
- `palette.rs`: Keyword detection (warm/cool/neon/nature) → curated color palettes for fallback

### tui/ — Ratatui Fullscreen TUI

- Alt screen mode, crossterm event loop
- Layout: ASCII title "BITART" top, pixel canvas center (scaled `█` blocks), input bar bottom, status bar
- States: Idle (input mode), Generating (spinner, blocked), Ready (showing art)
- Keybinds: Enter=generate, s=save PNG, r=regenerate, q=quit

### exporter/ — PNG Export

- Uses `image` crate, 64x64 pixels scaled 8x → 512x512 PNG
- Saves to `./output.png`, confirmation in status bar

## Dependencies

- ratatui + crossterm (TUI)
- image (PNG export)
- serde_json (parse Claude output)

## Error Handling

- Claude not in PATH → immediate fallback
- Claude returns invalid JSON → fallback
- Claude timeout (60s) → fallback
- PNG save failure → error message in status bar

## Excluded (YAGNI)

No config file, no CLI args, no canvas size options, no color picker, no undo/history, no async runtime.
