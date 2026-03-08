use std::io::{Cursor, Read};

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

/// Convert an image to a Canvas at its native resolution (no resize).
fn image_to_canvas(img: &image::DynamicImage) -> Canvas {
    let rgb = img.to_rgb8();
    let w = rgb.width();
    let h = rgb.height();

    let mut canvas: Canvas = Vec::with_capacity(h as usize);
    for y in 0..h {
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
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

/// Encode a DynamicImage to PNG bytes.
fn image_to_png_bytes(img: &image::DynamicImage) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    Ok(buf.into_inner())
}

/// Call OpenAI image edits API with reference images (multipart form).
fn call_edits_api(
    prompt: &str,
    api_key: &str,
    model: &str,
    reference_images: &[Vec<u8>],
) -> Result<image::DynamicImage, String> {
    let boundary = "----BitArtBoundary9876543210";
    let mut body: Vec<u8> = Vec::new();

    // Add each reference image
    for (i, png_bytes) in reference_images.iter().enumerate() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"image[]\"; filename=\"frame{}.png\"\r\n",
                i
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
        body.extend_from_slice(png_bytes);
        body.extend_from_slice(b"\r\n");
    }

    // Add prompt
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"prompt\"\r\n\r\n");
    body.extend_from_slice(prompt.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add model
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"model\"\r\n\r\n");
    body.extend_from_slice(model.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add size
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"size\"\r\n\r\n");
    body.extend_from_slice(b"1024x1024\r\n");

    // Close boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let response = ureq::post("https://api.openai.com/v1/images/edits")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
        .map_err(|e| format!("Edits API request failed: {}", e))?;

    let resp_body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse edits API response: {}", e))?;

    if let Some(err) = resp_body["error"]["message"].as_str() {
        return Err(format!("API error: {}", err));
    }

    // GPT Image models return base64
    if let Some(b64) = resp_body["data"][0]["b64_json"].as_str() {
        return decode_base64_image(b64);
    }
    // Fallback to URL
    if let Some(url) = resp_body["data"][0]["url"].as_str() {
        return download_image(url);
    }
    Err("No image data in edits response".to_string())
}

/// Get the appropriate image size for single image generation.
fn single_size(_model: &str) -> &'static str {
    "1024x1024"
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


/// Generate 3 animation frames using iterative edits API.
/// Call 1: generate base frame. Call 2: edit with frame 1 as reference. Call 3: edit with frames 1+2.
pub fn generate_spritesheet(prompt: &str, api_key: &str, model: &str) -> Result<Vec<Canvas>, String> {
    // Frame 1: generate the base image
    let base_prompt = format!(
        "Pixel art, 8-bit retro style, clear shapes with black outlines, vibrant colors, \
         game sprite style, plain white background, centered character: {}",
        prompt
    );

    let img1 = call_api(&base_prompt, api_key, model, single_size(model))?;
    let png1 = image_to_png_bytes(&img1)?;
    let canvas1 = image_to_canvas(&img1);

    // For non-GPT Image models, fall back to using the same image 3 times
    // since the edits API with image[] only works with GPT Image models
    if !is_gpt_image(model) {
        return Ok(vec![canvas1.clone(), canvas1.clone(), canvas1]);
    }

    // Frame 2: send frame 1 as reference, ask for subtle animation change
    let edit_prompt_2 = format!(
        "This is frame 1 of a pixel art animation of: {}. \
         Create frame 2 with ONE very subtle change — the character must stay in the EXACT same position, \
         same size, same outline. Only change a tiny detail that makes sense for the subject \
         (like a slight arm shift or eye blink). Keep the white background. Keep everything else identical.",
        prompt
    );

    let img2 = call_edits_api(&edit_prompt_2, api_key, model, &[png1.clone()])?;
    let png2 = image_to_png_bytes(&img2)?;
    let canvas2 = image_to_canvas(&img2);

    // Frame 3: send frames 1 and 2 as reference, ask for third frame
    let edit_prompt_3 = format!(
        "These are frames 1 and 2 of a pixel art animation of: {}. \
         Create frame 3 that completes the animation loop. The character must stay in the EXACT same position. \
         Make a subtle change that flows from frame 2 back toward frame 1 (like returning to rest position). \
         Keep the white background. Keep everything else identical.",
        prompt
    );

    let img3 = call_edits_api(&edit_prompt_3, api_key, model, &[png1, png2])?;
    let canvas3 = image_to_canvas(&img3);

    Ok(vec![canvas1, canvas2, canvas3])
}
