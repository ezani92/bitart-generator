use super::Canvas;
use crate::generator::palette::palette_for_prompt;

/// Simple hash function for seeding from a string.
fn hash_prompt(prompt: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in prompt.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// Generate a 64x64 canvas using math patterns seeded by the prompt.
pub fn generate(prompt: &str) -> Canvas {
    generate_with_seed(prompt, hash_prompt(prompt))
}

/// Generate with a specific seed (used for regeneration).
pub fn generate_with_seed(prompt: &str, seed: u64) -> Canvas {
    let palette = palette_for_prompt(prompt);
    let palette_len = palette.len();
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];

    let freq1 = ((seed % 7) as f64 + 1.0) * 0.15;
    let freq2 = ((seed % 11) as f64 + 1.0) * 0.1;
    let phase = (seed % 100) as f64 * 0.1;

    for y in 0usize..64 {
        for x in 0usize..64 {
            let xf = x as f64;
            let yf = y as f64;

            // Combine sine waves with XOR patterns
            let sine_val = (xf * freq1 + phase).sin() + (yf * freq2 + phase).cos();
            let xor_val = ((x ^ y) as f64 * 0.05 + seed as f64 * 0.001).sin();
            let combined = sine_val + xor_val;

            // Additional pattern from bitwise ops
            let bit_pattern = (x.wrapping_mul(seed as usize) ^ y.wrapping_mul(seed as usize >> 8)) % 256;
            let pattern_influence = bit_pattern as f64 / 256.0;

            let value = (combined + pattern_influence) * (palette_len as f64 / 4.0);
            let index = ((value.abs() as usize) % palette_len).min(palette_len - 1);

            canvas[y][x] = palette[index];
        }
    }

    canvas
}
