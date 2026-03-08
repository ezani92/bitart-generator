pub mod dalle;

use std::sync::mpsc;
use std::thread;

/// A 64x64 grid of RGB colors.
pub type Canvas = Vec<Vec<[u8; 3]>>;

/// Result of a generation attempt.
pub struct GenerationResult {
    pub canvas: Canvas,
    pub model: String,
}

/// Spawn generation in a background thread, returning a receiver for the result.
pub fn generate_async(
    prompt: String,
    api_key: String,
    model: String,
) -> mpsc::Receiver<Result<GenerationResult, String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = dalle::generate(&prompt, &api_key, &model);
        let _ = tx.send(result);
    });
    rx
}
