/// Returns a color palette based on keywords found in the prompt.
pub fn palette_for_prompt(prompt: &str) -> Vec<[u8; 3]> {
    let lower = prompt.to_lowercase();

    if lower.contains("neon") || lower.contains("cyber") || lower.contains("glow") {
        vec![
            [255, 0, 255],   // magenta
            [0, 255, 255],   // cyan
            [255, 255, 0],   // yellow
            [0, 255, 0],     // green
            [255, 0, 128],   // hot pink
            [0, 128, 255],   // blue
            [0, 0, 0],       // black background
            [32, 32, 32],    // dark gray
        ]
    } else if lower.contains("warm") || lower.contains("fire") || lower.contains("sunset") || lower.contains("lava") {
        vec![
            [255, 69, 0],    // red-orange
            [255, 140, 0],   // dark orange
            [255, 215, 0],   // gold
            [178, 34, 34],   // firebrick
            [255, 99, 71],   // tomato
            [139, 0, 0],     // dark red
            [255, 165, 0],   // orange
            [64, 0, 0],      // very dark red
        ]
    } else if lower.contains("cool") || lower.contains("ice") || lower.contains("ocean") || lower.contains("water") {
        vec![
            [0, 105, 148],   // deep blue
            [0, 191, 255],   // deep sky blue
            [135, 206, 250], // light sky blue
            [70, 130, 180],  // steel blue
            [0, 0, 139],     // dark blue
            [173, 216, 230], // light blue
            [240, 248, 255], // alice blue
            [0, 64, 128],    // navy variant
        ]
    } else if lower.contains("nature") || lower.contains("forest") || lower.contains("tree") || lower.contains("grass") {
        vec![
            [34, 139, 34],   // forest green
            [0, 100, 0],     // dark green
            [144, 238, 144], // light green
            [85, 107, 47],   // dark olive
            [139, 69, 19],   // saddle brown
            [160, 82, 45],   // sienna
            [107, 142, 35],  // olive drab
            [34, 60, 34],    // very dark green
        ]
    } else {
        // Default: diverse rainbow palette
        vec![
            [255, 87, 87],   // coral red
            [255, 189, 46],  // amber
            [46, 213, 115],  // emerald
            [30, 144, 255],  // dodger blue
            [155, 89, 182],  // amethyst
            [255, 255, 255], // white
            [52, 73, 94],    // dark slate
            [0, 0, 0],       // black
        ]
    }
}
