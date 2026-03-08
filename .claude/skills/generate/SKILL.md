---
name: generate
description: Generate pixel art using BitArt CLI from a text prompt. Use when the user asks to create, generate, or make pixel art, sprites, or game assets.
argument-hint: <prompt> [gif] [-o path]
allowed-tools: Bash
---

Generate pixel art using the `bitart` CLI tool.

## Parse Arguments

From `$ARGUMENTS`, extract:
1. **prompt** (required) — The text description of what to generate
2. **gif mode** — If the user includes "gif", "animated", or "animation" in the arguments
3. **output path** — If the user specifies `-o <path>` or a file path ending in `.png` or `.gif`

## Commands

### PNG (default)
```bash
bitart -p "<prompt>"
```

### PNG with custom output
```bash
bitart -p "<prompt>" -o "<path>"
```

### Animated GIF
```bash
bitart -p "<prompt>" -g
```

### Animated GIF with custom output
```bash
bitart -p "<prompt>" -g -o "<path>"
```

## Rules

- Strip "gif", "animated", "animation" from the prompt text before passing to `-p`
- Strip `-o <path>` from the prompt text before passing to `-p`
- Always quote the prompt with double quotes
- GIF mode makes 3 API calls (3x cost) — mention this to the user
- Output defaults to the configured save folder with a timestamped filename
- After generation, tell the user the output path shown in stderr

## Examples

- `/generate a dragon breathing fire` → `bitart -p "a dragon breathing fire"`
- `/generate skeleton knight gif` → `bitart -p "skeleton knight" -g`
- `/generate cute cat -o cat.png` → `bitart -p "cute cat" -o "cat.png"`
- `/generate dancing robot animated -o robot.gif` → `bitart -p "dancing robot" -g -o "robot.gif"`
