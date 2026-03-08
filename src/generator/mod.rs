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

/// A named tile from chained generation.
pub struct ChainTile {
    pub name: String,
    pub canvas: Canvas,
}

/// Result of chained asset generation.
pub struct ChainResult {
    pub tiles: Vec<ChainTile>,
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

/// Spawn chained asset generation — text API + multiple image API calls.
pub fn generate_chain_async(
    prompt: String,
    api_key: String,
    model: String,
) -> mpsc::Receiver<Result<ChainResult, String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        match dalle::generate_chain(&prompt, &api_key, &model) {
            Ok(tiles) => {
                let chain_tiles = tiles
                    .into_iter()
                    .map(|t| ChainTile {
                        name: t.name,
                        canvas: t.canvas,
                    })
                    .collect();
                let _ = tx.send(Ok(ChainResult {
                    tiles: chain_tiles,
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
