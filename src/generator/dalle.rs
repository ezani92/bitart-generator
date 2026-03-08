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

/// Decode a base64-encoded image.
fn decode_base64_image(b64: &str) -> Result<image::DynamicImage, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

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

/// Check if model is GPT Image series.
fn is_gpt_image(model: &str) -> bool {
    model.starts_with("gpt-image")
}

/// Call OpenAI image generation API.
fn call_api(prompt: &str, api_key: &str, model: &str, size: &str) -> Result<image::DynamicImage, String> {
    // GPT Image models don't support response_format parameter
    let json = if is_gpt_image(model) {
        serde_json::json!({
            "model": model,
            "prompt": prompt,
            "n": 1,
            "size": size,
        })
    } else {
        serde_json::json!({
            "model": model,
            "prompt": prompt,
            "n": 1,
            "size": size,
            "response_format": "url",
        })
    };

    let response = ureq::post("https://api.openai.com/v1/images/generations")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(json)
        .map_err(|e| format!("API request failed: {}", e))?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    // Check for error
    if let Some(err) = body["error"]["message"].as_str() {
        return Err(format!("API error: {}", err));
    }

    if is_gpt_image(model) {
        let b64 = body["data"][0]["b64_json"]
            .as_str()
            .ok_or_else(|| "No base64 image data in response".to_string())?;
        decode_base64_image(b64)
    } else {
        let url = body["data"][0]["url"]
            .as_str()
            .ok_or_else(|| "No image URL in response".to_string())?;
        download_image(url)
    }
}

/// Get the appropriate image size for single image generation.
fn single_size(_model: &str) -> &'static str {
    "1024x1024"
}

/// Get the appropriate wide image size for sprite sheet generation.
fn spritesheet_size(model: &str) -> &'static str {
    if model == "dall-e-2" {
        "1024x1024"
    } else if is_gpt_image(model) {
        "1536x1024"
    } else {
        // dall-e-3
        "1792x1024"
    }
}

/// Generate a single pixel art image.
pub fn generate(prompt: &str, api_key: &str, model: &str) -> Result<GenerationResult, String> {
    let full_prompt = format!(
        "Pixel art, 8-bit retro style, clear shapes with black outlines, vibrant colors, game sprite style: {}",
        prompt
    );

    let img = call_api(&full_prompt, api_key, model, single_size(model))?;
    let canvas = image_to_canvas(&img);

    Ok(GenerationResult {
        canvas,
        model: model.to_string(),
    })
}

/// Find the bounding box of non-background pixels in a canvas.
/// Returns (min_x, min_y, max_x, max_y) or None if empty.
fn find_bounds(canvas: &Canvas, bg: [u8; 3], tolerance: u8) -> Option<(usize, usize, usize, usize)> {
    let mut min_x = 64usize;
    let mut min_y = 64usize;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    let mut found = false;

    for (y, row) in canvas.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            let dr = (pixel[0] as i16 - bg[0] as i16).unsigned_abs() as u8;
            let dg = (pixel[1] as i16 - bg[1] as i16).unsigned_abs() as u8;
            let db = (pixel[2] as i16 - bg[2] as i16).unsigned_abs() as u8;
            if dr > tolerance || dg > tolerance || db > tolerance {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if found { Some((min_x, min_y, max_x, max_y)) } else { None }
}

/// Shift a canvas so its subject center aligns with the target center.
fn align_canvas(canvas: &Canvas, bg: [u8; 3], target_cx: i32, target_cy: i32, tolerance: u8) -> Canvas {
    let bounds = match find_bounds(canvas, bg, tolerance) {
        Some(b) => b,
        None => return canvas.clone(),
    };

    let cx = (bounds.0 as i32 + bounds.2 as i32) / 2;
    let cy = (bounds.1 as i32 + bounds.3 as i32) / 2;

    let dx = target_cx - cx;
    let dy = target_cy - cy;

    if dx == 0 && dy == 0 {
        return canvas.clone();
    }

    let mut result = vec![vec![bg; 64]; 64];
    for y in 0..64i32 {
        for x in 0..64i32 {
            let src_x = x - dx;
            let src_y = y - dy;
            if src_x >= 0 && src_x < 64 && src_y >= 0 && src_y < 64 {
                result[y as usize][x as usize] = canvas[src_y as usize][src_x as usize];
            }
        }
    }
    result
}

/// Generate a sprite sheet with 3 animation frames, split, and auto-align.
pub fn generate_spritesheet(prompt: &str, api_key: &str, model: &str) -> Result<Vec<Canvas>, String> {
    let full_prompt = format!(
        "A horizontal pixel art sprite sheet with exactly 3 animation frames placed side by side, separated by thin vertical lines. \
         8-bit retro style, clear shapes with black outlines, vibrant colors. \
         Subject: {}. \
         IMPORTANT: The main body/structure must stay in the EXACT same position and shape across all 3 frames. \
         Only animate small subtle details (e.g. leaves swaying, eyes blinking, tail wagging, flames flickering, water rippling). \
         The background, position, size, colors, and overall composition must be IDENTICAL in all 3 frames. \
         The difference between frames should be very small and subtle, like a 2-3 pixel shift on moving parts only. \
         White background.",
        prompt
    );

    let img = call_api(&full_prompt, api_key, model, spritesheet_size(model))?;

    // Split image into 3 equal horizontal panels
    let width = img.width();
    let height = img.height();
    let frame_width = width / 3;

    let mut frames: Vec<Canvas> = Vec::with_capacity(3);
    for i in 0..3 {
        let x = i * frame_width;
        let cropped = img.crop_imm(x, 0, frame_width, height);
        let canvas = image_to_canvas(&cropped);
        frames.push(canvas);
    }

    // Auto-align: use frame 1 as reference, align frames 2 & 3 to match
    let bg = frames[0][0][0]; // top-left pixel as background
    let tolerance = 30u8;

    if let Some(ref_bounds) = find_bounds(&frames[0], bg, tolerance) {
        let target_cx = (ref_bounds.0 as i32 + ref_bounds.2 as i32) / 2;
        let target_cy = (ref_bounds.1 as i32 + ref_bounds.3 as i32) / 2;

        frames[1] = align_canvas(&frames[1], bg, target_cx, target_cy, tolerance);
        frames[2] = align_canvas(&frames[2], bg, target_cx, target_cy, tolerance);
    }

    Ok(frames)
}
