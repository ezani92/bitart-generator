use super::Canvas;
use std::process::Command;

/// Attempt to generate pixel art by shelling out to the claude CLI.
/// Generates a 32x32 grid and upscales to 64x64 for speed.
pub fn generate(prompt: &str) -> Result<Canvas, String> {
    let claude_prompt = format!(
        "You are a pixel art generator. Return ONLY a valid JSON array of 32 arrays, \
         each containing 32 hex color strings (#RRGGBB). No explanation, no markdown, \
         raw JSON only. Make it look like classic 8-bit pixel art with clear shapes and outlines. \
         Generate pixel art for: {}",
        prompt
    );

    let output = Command::new("claude")
        .arg("-p")
        .arg(&claude_prompt)
        .arg("--output-format")
        .arg("text")
        .output()
        .map_err(|e| format!("Failed to run claude: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude exited with error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let small = parse_claude_output(&stdout)?;
    Ok(upscale_2x(&small))
}

/// Parse the JSON output from claude into a 32x32 Canvas.
fn parse_claude_output(raw: &str) -> Result<Canvas, String> {
    let trimmed = raw.trim();
    let json_start = trimmed.find('[').ok_or("No JSON array found in output")?;
    // Find matching closing bracket
    let json_str = find_json_array(&trimmed[json_start..])?;

    let parsed: Vec<Vec<String>> =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    if parsed.len() != 32 {
        return Err(format!("Expected 32 rows, got {}", parsed.len()));
    }

    let mut canvas: Canvas = Vec::with_capacity(32);
    for (y, row) in parsed.iter().enumerate() {
        if row.len() != 32 {
            return Err(format!("Row {} has {} columns, expected 32", y, row.len()));
        }
        let mut canvas_row = Vec::with_capacity(32);
        for hex in row {
            let color = parse_hex_color(hex)
                .ok_or_else(|| format!("Invalid hex color '{}' at row {}", hex, y))?;
            canvas_row.push(color);
        }
        canvas.push(canvas_row);
    }

    Ok(canvas)
}

/// Find the complete JSON array by matching brackets.
fn find_json_array(s: &str) -> Result<&str, String> {
    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if end == 0 {
        return Err("No complete JSON array found".into());
    }
    Ok(&s[..end])
}

/// Upscale a 32x32 canvas to 64x64 (2x nearest-neighbor).
fn upscale_2x(small: &Canvas) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    for y in 0..64 {
        for x in 0..64 {
            canvas[y][x] = small[y / 2][x / 2];
        }
    }
    canvas
}

/// Parse a hex color string like "#FF00AA" into [r, g, b].
fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}
