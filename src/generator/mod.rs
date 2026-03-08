pub mod dalle;

use std::sync::mpsc;
use std::thread;

/// A grid of RGB colors.
pub type Canvas = Vec<Vec<[u8; 3]>>;

/// Result of a generation attempt.
pub struct GenerationResult {
    pub canvas: Canvas,
    pub model: String,
}

/// Result of multi-frame generation.
pub struct FramesResult {
    pub frames: Vec<Canvas>,
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

/// Spawn sprite sheet generation (for GIF) — one API call, 3 frames.
pub fn generate_frames_async(
    prompt: String,
    api_key: String,
    model: String,
) -> mpsc::Receiver<Result<FramesResult, String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        match dalle::generate_spritesheet(&prompt, &api_key, &model) {
            Ok(frames) => {
                let _ = tx.send(Ok(FramesResult {
                    frames,
                    model: model.to_string(),
                }));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });
    rx
}
