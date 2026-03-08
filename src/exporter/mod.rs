use crate::generator::Canvas;
use image::RgbaImage;

const WHITE_THRESHOLD: u8 = 240;

/// Save the 64x64 canvas as a 512x512 PNG (8x scale).
/// Near-white pixels are exported as transparent.
pub fn save_png(canvas: &Canvas, path: &str) -> Result<(), String> {
    let scale: u32 = 8;
    let size = 64 * scale; // 512
    let mut img = RgbaImage::new(size, size);

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

    img.save(path).map_err(|e| format!("Failed to save PNG: {}", e))
}
