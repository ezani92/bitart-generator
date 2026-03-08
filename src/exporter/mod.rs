use crate::generator::Canvas;
use image::RgbImage;

/// Save the 64x64 canvas as a 512x512 PNG (8x scale).
pub fn save_png(canvas: &Canvas, path: &str) -> Result<(), String> {
    let scale: u32 = 8;
    let size = 64 * scale; // 512
    let mut img = RgbImage::new(size, size);

    for (y, row) in canvas.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x as u32 * scale + dx;
                    let py = y as u32 * scale + dy;
                    img.put_pixel(px, py, image::Rgb(*color));
                }
            }
        }
    }

    img.save(path).map_err(|e| format!("Failed to save PNG: {}", e))
}
