pub mod claude;
pub mod fallback;
pub mod palette;

use std::sync::mpsc;
use std::thread;

/// A 64x64 grid of RGB colors.
pub type Canvas = Vec<Vec<[u8; 3]>>;

/// Result of a generation attempt.
pub struct GenerationResult {
    pub canvas: Canvas,
    pub mode: GenerationMode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GenerationMode {
    Ai,
    Fallback,
}

impl std::fmt::Display for GenerationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationMode::Ai => write!(f, "AI"),
            GenerationMode::Fallback => write!(f, "Fallback"),
        }
    }
}

/// Generate pixel art from a prompt. Tries Claude first, falls back to math.
pub fn generate(prompt: &str, seed: u64) -> GenerationResult {
    match claude::generate(prompt) {
        Ok(canvas) => GenerationResult {
            canvas,
            mode: GenerationMode::Ai,
        },
        Err(_) => GenerationResult {
            canvas: fallback::generate_with_seed(prompt, seed),
            mode: GenerationMode::Fallback,
        },
    }
}

/// Spawn generation in a background thread, returning a receiver for the result.
pub fn generate_async(prompt: String, seed: u64) -> mpsc::Receiver<GenerationResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = generate(&prompt, seed);
        let _ = tx.send(result);
    });
    rx
}
