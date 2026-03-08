use std::io::Read;

use super::{Canvas, GenerationResult};

/// Generate pixel art using DALL-E API.
pub fn generate(prompt: &str, api_key: &str, model: &str) -> Result<GenerationResult, String> {
    let full_prompt = format!(
        "Pixel art, 8-bit retro style, clear shapes with black outlines, vibrant colors, game sprite style: {}",
        prompt
    );

    // Call DALL-E API
    let response = ureq::post("https://api.openai.com/v1/images/generations")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": model,
            "prompt": full_prompt,
            "n": 1,
            "size": "1024x1024",
            "response_format": "url"
        }))
        .map_err(|e| format!("API request failed: {}", e))?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    // Extract image URL
    let url = body["data"][0]["url"]
        .as_str()
        .ok_or_else(|| {
            // Check for error message
            if let Some(err) = body["error"]["message"].as_str() {
                format!("API error: {}", err)
            } else {
                "No image URL in response".to_string()
            }
        })?;

    // Download the image
    let img_response = ureq::get(url)
        .call()
        .map_err(|e| format!("Failed to download image: {}", e))?;

    let mut bytes: Vec<u8> = Vec::new();
    img_response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read image data: {}", e))?;

    // Decode and resize to 64x64
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let resized = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);
    let rgb = resized.to_rgb8();

    // Convert to Canvas
    let mut canvas: Canvas = Vec::with_capacity(64);
    for y in 0..64 {
        let mut row = Vec::with_capacity(64);
        for x in 0..64 {
            let pixel = rgb.get_pixel(x, y);
            row.push([pixel[0], pixel[1], pixel[2]]);
        }
        canvas.push(row);
    }

    Ok(GenerationResult {
        canvas,
        model: model.to_string(),
    })
}
