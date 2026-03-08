use std::io::Read;

use super::{Canvas, GenerationResult};

/// Download and decode an image from a URL.
fn download_image(url: &str) -> Result<image::DynamicImage, String> {
    let img_response = ureq::get(url)
        .call()
        .map_err(|e| format!("Failed to download image: {}", e))?;

    let mut bytes: Vec<u8> = Vec::new();
    img_response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read image data: {}", e))?;

    image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))
}

/// Convert an image to a 64x64 Canvas.
fn image_to_canvas(img: &image::DynamicImage) -> Canvas {
    let resized = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);
    let rgb = resized.to_rgb8();

    let mut canvas: Canvas = Vec::with_capacity(64);
    for y in 0..64 {
        let mut row = Vec::with_capacity(64);
        for x in 0..64 {
            let pixel = rgb.get_pixel(x, y);
            row.push([pixel[0], pixel[1], pixel[2]]);
        }
        canvas.push(row);
    }
    canvas
}

/// Call DALL-E API and return the image URL.
fn call_dalle(prompt: &str, api_key: &str, model: &str, size: &str) -> Result<String, String> {
    let response = ureq::post("https://api.openai.com/v1/images/generations")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": model,
            "prompt": prompt,
            "n": 1,
            "size": size,
            "response_format": "url"
        }))
        .map_err(|e| format!("API request failed: {}", e))?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    body["data"][0]["url"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            if let Some(err) = body["error"]["message"].as_str() {
                format!("API error: {}", err)
            } else {
                "No image URL in response".to_string()
            }
        })
}

/// Generate a single pixel art image.
pub fn generate(prompt: &str, api_key: &str, model: &str) -> Result<GenerationResult, String> {
    let full_prompt = format!(
        "Pixel art, 8-bit retro style, clear shapes with black outlines, vibrant colors, game sprite style: {}",
        prompt
    );

    let url = call_dalle(&full_prompt, api_key, model, "1024x1024")?;
    let img = download_image(&url)?;
    let canvas = image_to_canvas(&img);

    Ok(GenerationResult {
        canvas,
        model: model.to_string(),
    })
}

/// Generate a sprite sheet with 3 animation frames in one image, then split into 3 canvases.
pub fn generate_spritesheet(prompt: &str, api_key: &str, model: &str) -> Result<Vec<Canvas>, String> {
    let full_prompt = format!(
        "A horizontal sprite sheet showing exactly 3 animation frames side by side of the same character. \
         Pixel art, 8-bit retro style, clear shapes with black outlines, vibrant colors, game sprite style. \
         The 3 frames show a smooth looping animation sequence of: {}. \
         Each frame is separated by a thin vertical line. Same character, same colors, same size in all 3 frames. \
         White background.",
        prompt
    );

    let size = if model == "dall-e-3" {
        "1792x1024"
    } else {
        "1024x1024"
    };

    let url = call_dalle(&full_prompt, api_key, model, size)?;
    let img = download_image(&url)?;

    // Split image into 3 equal horizontal panels
    let width = img.width();
    let height = img.height();
    let frame_width = width / 3;

    let mut frames = Vec::with_capacity(3);
    for i in 0..3 {
        let x = i * frame_width;
        let cropped = img.crop_imm(x, 0, frame_width, height);
        let canvas = image_to_canvas(&cropped);
        frames.push(canvas);
    }

    Ok(frames)
}
