---
name: generate-pixel-art
description: Generate pixel art using BitArt CLI from a text prompt. Use when the user asks to create, generate, or make pixel art, sprites, or game assets.
---

# Generate Pixel Art with BitArt

BitArt is a terminal-based pixel art generator that uses OpenAI image generation APIs. It is installed as the `bitart` CLI tool.

## CLI Usage

### Generate PNG
```bash
bitart -p "<prompt>"
```

### Generate PNG with custom output path
```bash
bitart -p "<prompt>" -o "<path>"
```

### Generate animated GIF (3 frames at 3fps)
```bash
bitart -p "<prompt>" -g
```

### Generate animated GIF with custom output path
```bash
bitart -p "<prompt>" -g -o "<path>"
```

### Other flags
```bash
bitart -h          # Show help
bitart -v          # Show version
```

## Notes

- Requires `bitart` to be installed: `npm install -g bitart-generator` or `cargo install bitart-generator`
- Requires OpenAI API key configured via `bitart` first-run setup (saved at `~/.bitart/config.json`)
- Supports 5 models: DALL-E 2, DALL-E 3, GPT Image Mini, GPT Image 1, GPT Image 1.5
- PNG exports have transparent backgrounds (near-white pixels become transparent)
- GIF mode makes 3 API calls using the edits API for contextually aware frames (3x cost)
- Output defaults to configured save folder (`~/Downloads/bitart/`) with timestamped filenames
- Images are generated at native API resolution (1024x1024)

## Examples

| User request | Command |
|---|---|
| "Generate a dragon" | `bitart -p "a dragon"` |
| "Make a skeleton knight sprite as GIF" | `bitart -p "skeleton knight" -g` |
| "Create pixel art cat, save to cat.png" | `bitart -p "cute cat" -o "cat.png"` |
| "Animated robot, save as robot.gif" | `bitart -p "dancing robot" -g -o "robot.gif"` |
