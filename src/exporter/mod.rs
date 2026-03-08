use crate::generator::Canvas;
use image::{codecs::gif::GifEncoder, Frame, RgbaImage};
use std::fs::File;

const WHITE_THRESHOLD: u8 = 240;

fn canvas_to_rgba(canvas: &Canvas, scale: u32) -> RgbaImage {
    let h = canvas.len() as u32;
    let w = if h > 0 { canvas[0].len() as u32 } else { 0 };
    let mut img = RgbaImage::new(w * scale, h * scale);

    for (y, row) in canvas.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            let [r, g, b] = *color;
            let rgba = if r >= WHITE_THRESHOLD && g >= WHITE_THRESHOLD && b >= WHITE_THRESHOLD {
                image::Rgba([r, g, b, 0])
            } else {
                image::Rgba([r, g, b, 255])
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x as u32 * scale + dx;
                    let py = y as u32 * scale + dy;
                    img.put_pixel(px, py, rgba);
                }
            }
        }
    }

    img
}

/// Save the canvas as PNG at native resolution (1024x1024).
/// Near-white pixels are exported as transparent.
pub fn save_png(canvas: &Canvas, path: &str) -> Result<(), String> {
    let img = canvas_to_rgba(canvas, 1);
    img.save(path).map_err(|e| format!("Failed to save PNG: {}", e))
}

/// Save multiple named canvases as individual PNGs in a subfolder.
pub fn save_chain_pngs(
    tiles: &[(String, &Canvas)],
    base_dir: &str,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(base_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let mut paths = Vec::new();
    for (name, canvas) in tiles {
        // Sanitize name for filename
        let safe_name: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
            .collect();
        let filename = format!("{}.png", safe_name);
        let path = std::path::Path::new(base_dir)
            .join(&filename)
            .to_string_lossy()
            .to_string();

        let img = canvas_to_rgba(canvas, 1);
        img.save(&path)
            .map_err(|e| format!("Failed to save {}: {}", filename, e))?;
        paths.push(path);
    }

    Ok(paths)
}

/// Save multiple canvases as an animated GIF at 3fps (333ms per frame).
pub fn save_gif(frames: &[Canvas], path: &str) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut encoder = GifEncoder::new_with_speed(file, 10);
    encoder
        .set_repeat(image::codecs::gif::Repeat::Infinite)
        .map_err(|e| format!("Failed to set repeat: {}", e))?;

    for canvas in frames {
        let rgba = canvas_to_rgba(canvas, 1);
        let frame = Frame::from_parts(
            rgba,
            0,
            0,
            image::Delay::from_numer_denom_ms(333, 1),
        );
        encoder
            .encode_frame(frame)
            .map_err(|e| format!("Failed to encode frame: {}", e))?;
    }

    Ok(())
}
