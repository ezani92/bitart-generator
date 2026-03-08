pub mod claude;
pub mod fallback;
pub mod palette;

/// A 64x64 grid of RGB colors
pub type Canvas = Vec<Vec<[u8; 3]>>;

/// Generate pixel art from a prompt. Tries Claude first, falls back to math.
pub fn generate(prompt: &str) -> Canvas {
    match claude::generate(prompt) {
        Ok(canvas) => canvas,
        Err(_) => fallback::generate(prompt),
    }
}
