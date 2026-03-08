Generate pixel art using BitArt CLI.

Usage: /generate <prompt> [options]

Arguments:
- $ARGUMENTS — The prompt describing what to generate, optionally followed by flags

This command uses the `bitart` CLI tool to generate pixel art from text prompts.

## Instructions

Parse the user's arguments to extract:
1. **prompt** (required) — The text description of what to generate
2. **output path** (optional) — If the user specifies a path or filename
3. **gif mode** (optional) — If the user says "gif", "animated", or "animation"

Then run the appropriate `bitart` command:

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

## Examples

- `/generate a dragon breathing fire` → `bitart -p "a dragon breathing fire"`
- `/generate skeleton knight gif` → `bitart -p "skeleton knight" -g`
- `/generate cute cat -o cat.png` → `bitart -p "cute cat" -o "cat.png"`
- `/generate dancing robot animated -o robot.gif` → `bitart -p "dancing robot" -g -o "robot.gif"`

## Notes

- BitArt must be installed (`cargo install bitart-generator` or `npm install -g bitart-generator`)
- Requires OpenAI API key configured via `bitart` first-run setup
- PNG exports have transparent backgrounds (near-white pixels become transparent)
- GIF mode makes 3 API calls (costs 3x) but produces smooth animation
- Output defaults to configured save folder with timestamped filename if no -o specified
