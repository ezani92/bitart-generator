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

/// Spawn multi-frame generation (for GIF), returning a receiver for progress and final result.
pub fn generate_frames_async(
    prompt: String,
    api_key: String,
    model: String,
    frame_count: usize,
) -> mpsc::Receiver<Result<FramesResult, String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut frames = Vec::with_capacity(frame_count);
        for i in 0..frame_count {
            let frame_prompt = if i == 0 {
                prompt.clone()
            } else {
                format!("{} (variation {})", prompt, i + 1)
            };
            match dalle::generate(&frame_prompt, &api_key, &model) {
                Ok(result) => frames.push(result.canvas),
                Err(e) => {
                    let _ = tx.send(Err(format!("Frame {} failed: {}", i + 1, e)));
                    return;
                }
            }
        }
        let _ = tx.send(Ok(FramesResult {
            frames,
            model: model.to_string(),
        }));
    });
    rx
}
