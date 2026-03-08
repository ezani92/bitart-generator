use super::Canvas;
use std::process::Command;

/// Attempt to generate pixel art by shelling out to the claude CLI.
pub fn generate(prompt: &str) -> Result<Canvas, String> {
    let claude_prompt = format!(
        "You are a pixel art generator. Return ONLY a valid JSON array of 64 arrays, \
         each containing 64 hex color strings (#RRGGBB). No explanation, no markdown, \
         raw JSON only. Generate pixel art for: {}",
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
    parse_claude_output(&stdout)
}

/// Parse the JSON output from claude into a Canvas.
fn parse_claude_output(raw: &str) -> Result<Canvas, String> {
    let trimmed = raw.trim();
    let json_start = trimmed.find('[').ok_or("No JSON array found in output")?;
    let json_str = &trimmed[json_start..];

    let parsed: Vec<Vec<String>> =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    if parsed.len() != 64 {
        return Err(format!("Expected 64 rows, got {}", parsed.len()));
    }

    let mut canvas: Canvas = Vec::with_capacity(64);
    for (y, row) in parsed.iter().enumerate() {
        if row.len() != 64 {
            return Err(format!("Row {} has {} columns, expected 64", y, row.len()));
        }
        let mut canvas_row = Vec::with_capacity(64);
        for hex in row {
            let color = parse_hex_color(hex)
                .ok_or_else(|| format!("Invalid hex color '{}' at row {}", hex, y))?;
            canvas_row.push(color);
        }
        canvas.push(canvas_row);
    }

    Ok(canvas)
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
