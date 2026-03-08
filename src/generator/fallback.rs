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

/// Pseudo-random number generator (xorshift64).
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Returns a value in [0, max)
    fn next_range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }

    /// Returns a float in [0.0, 1.0)
    fn next_f64(&mut self) -> f64 {
        (self.next() % 10000) as f64 / 10000.0
    }
}

/// Generate a 64x64 canvas using math patterns seeded by the prompt.
pub fn generate(prompt: &str) -> Canvas {
    generate_with_seed(prompt, hash_prompt(prompt))
}

/// Generate with a specific seed (used for regeneration).
pub fn generate_with_seed(prompt: &str, seed: u64) -> Canvas {
    let palette = palette_for_prompt(prompt);
    let lower = prompt.to_lowercase();
    let mut rng = Rng::new(seed);

    // Choose scene type based on keywords
    if lower.contains("tree") || lower.contains("oak") || lower.contains("pine") {
        generate_tree(&palette, &mut rng)
    } else if lower.contains("mountain") || lower.contains("hill") {
        generate_mountains(&palette, &mut rng)
    } else if lower.contains("sunset") || lower.contains("sunrise") {
        generate_sunset(&palette, &mut rng)
    } else if lower.contains("ocean") || lower.contains("sea") || lower.contains("water") || lower.contains("wave") {
        generate_ocean(&palette, &mut rng)
    } else if lower.contains("city") || lower.contains("building") || lower.contains("skyline") {
        generate_city(&palette, &mut rng)
    } else if lower.contains("heart") || lower.contains("love") {
        generate_heart(&palette, &mut rng)
    } else if lower.contains("star") || lower.contains("space") || lower.contains("galaxy") {
        generate_stars(&palette, &mut rng)
    } else if lower.contains("face") || lower.contains("smiley") || lower.contains("emoji") {
        generate_face(&palette, &mut rng)
    } else if lower.contains("flower") || lower.contains("rose") || lower.contains("plant") {
        generate_flower(&palette, &mut rng)
    } else if lower.contains("sword") || lower.contains("weapon") || lower.contains("blade") {
        generate_sword(&palette, &mut rng)
    } else {
        generate_landscape(&palette, &mut rng)
    }
}

/// Fill background color.
fn fill(canvas: &mut Canvas, color: [u8; 3]) {
    for row in canvas.iter_mut() {
        for pixel in row.iter_mut() {
            *pixel = color;
        }
    }
}

/// Draw a filled circle.
fn draw_circle(canvas: &mut Canvas, cx: i32, cy: i32, r: i32, color: [u8; 3]) {
    for y in 0..64i32 {
        for x in 0..64i32 {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                if y >= 0 && y < 64 && x >= 0 && x < 64 {
                    canvas[y as usize][x as usize] = color;
                }
            }
        }
    }
}

/// Draw a filled rectangle.
fn draw_rect(canvas: &mut Canvas, x1: i32, y1: i32, x2: i32, y2: i32, color: [u8; 3]) {
    for y in y1.max(0)..y2.min(64) {
        for x in x1.max(0)..x2.min(64) {
            canvas[y as usize][x as usize] = color;
        }
    }
}

/// Scatter random pixels of a color in a region.
fn scatter(canvas: &mut Canvas, x1: i32, y1: i32, x2: i32, y2: i32, color: [u8; 3], density: f64, rng: &mut Rng) {
    for y in y1.max(0)..y2.min(64) {
        for x in x1.max(0)..x2.min(64) {
            if rng.next_f64() < density {
                canvas[y as usize][x as usize] = color;
            }
        }
    }
}

fn generate_tree(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let sky = [135, 206, 235];
    let grass_color = [34, 139, 34];
    let trunk = [101, 67, 33];
    let dark_trunk = [72, 45, 20];
    let leaf1 = palette.get(0).copied().unwrap_or([34, 139, 34]);
    let leaf2 = palette.get(1).copied().unwrap_or([0, 100, 0]);
    let leaf3 = palette.get(2).copied().unwrap_or([50, 180, 50]);

    // Sky
    fill(&mut canvas, sky);

    // Ground
    draw_rect(&mut canvas, 0, 50, 64, 64, grass_color);
    scatter(&mut canvas, 0, 50, 64, 64, [40, 160, 40], 0.3, rng);
    scatter(&mut canvas, 0, 50, 64, 64, [28, 120, 28], 0.2, rng);

    // Trunk
    let trunk_x = 28 + rng.next_range(8) as i32;
    draw_rect(&mut canvas, trunk_x, 25, trunk_x + 8, 52, trunk);
    draw_rect(&mut canvas, trunk_x + 1, 25, trunk_x + 3, 52, dark_trunk);

    // Canopy - layered circles for tree crown
    let cx = trunk_x + 4;
    draw_circle(&mut canvas, cx, 22, 14, leaf1);
    draw_circle(&mut canvas, cx - 6, 20, 10, leaf2);
    draw_circle(&mut canvas, cx + 7, 18, 10, leaf2);
    draw_circle(&mut canvas, cx, 14, 11, leaf3);
    draw_circle(&mut canvas, cx - 4, 24, 8, leaf2);
    draw_circle(&mut canvas, cx + 5, 22, 9, leaf1);

    // Leaf texture
    scatter(&mut canvas, cx - 16, 6, cx + 16, 36, leaf2, 0.15, rng);
    scatter(&mut canvas, cx - 14, 8, cx + 14, 34, leaf3, 0.1, rng);

    canvas
}

fn generate_mountains(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let sky_top = [100, 149, 237];
    let sky_bot = [173, 216, 230];

    // Gradient sky
    for y in 0..40 {
        let t = y as f64 / 40.0;
        let r = (sky_top[0] as f64 * (1.0 - t) + sky_bot[0] as f64 * t) as u8;
        let g = (sky_top[1] as f64 * (1.0 - t) + sky_bot[1] as f64 * t) as u8;
        let b = (sky_top[2] as f64 * (1.0 - t) + sky_bot[2] as f64 * t) as u8;
        for x in 0..64 {
            canvas[y][x] = [r, g, b];
        }
    }

    // Sun
    let sun_x = 10 + rng.next_range(20) as i32;
    draw_circle(&mut canvas, sun_x, 10, 5, [255, 223, 100]);

    let mt1 = palette.get(0).copied().unwrap_or([100, 100, 120]);
    let mt2 = palette.get(1).copied().unwrap_or([80, 80, 100]);
    let snow = [240, 240, 255];
    let grass = [34, 120, 34];

    // Background mountain
    let peak1 = 15 + rng.next_range(10) as i32;
    for x in 0..64i32 {
        let dx = (x - 20).abs() as f64;
        let h = (peak1 as f64 + dx * 0.7) as i32;
        for y in h.max(0)..55 {
            if y < 64 {
                canvas[y as usize][x as usize] = mt2;
            }
        }
        if h < peak1 + 5 {
            for y in h.max(0)..(h + 3).min(64) {
                canvas[y as usize][x as usize] = snow;
            }
        }
    }

    // Foreground mountain
    let peak2 = 20 + rng.next_range(8) as i32;
    for x in 0..64i32 {
        let dx = (x - 44).abs() as f64;
        let h = (peak2 as f64 + dx * 0.6) as i32;
        for y in h.max(0)..58 {
            if y < 64 {
                canvas[y as usize][x as usize] = mt1;
            }
        }
        if h < peak2 + 4 {
            for y in h.max(0)..(h + 3).min(64) {
                canvas[y as usize][x as usize] = snow;
            }
        }
    }

    // Green ground
    draw_rect(&mut canvas, 0, 55, 64, 64, grass);
    scatter(&mut canvas, 0, 55, 64, 64, [28, 100, 28], 0.3, rng);

    canvas
}

fn generate_sunset(_palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];

    // Gradient sky: purple -> orange -> yellow
    let colors: Vec<[u8; 3]> = vec![
        [75, 0, 130],    // top: deep purple
        [148, 0, 115],   // purple-pink
        [255, 69, 0],    // red-orange
        [255, 140, 0],   // orange
        [255, 200, 50],  // yellow-gold
        [255, 230, 120], // light yellow at horizon
    ];

    for y in 0..40 {
        let t = y as f64 / 40.0 * (colors.len() - 1) as f64;
        let idx = t as usize;
        let frac = t - idx as f64;
        let c1 = colors[idx.min(colors.len() - 1)];
        let c2 = colors[(idx + 1).min(colors.len() - 1)];
        let r = (c1[0] as f64 * (1.0 - frac) + c2[0] as f64 * frac) as u8;
        let g = (c1[1] as f64 * (1.0 - frac) + c2[1] as f64 * frac) as u8;
        let b = (c1[2] as f64 * (1.0 - frac) + c2[2] as f64 * frac) as u8;
        for x in 0..64 {
            canvas[y][x] = [r, g, b];
        }
    }

    // Sun
    draw_circle(&mut canvas, 32, 34, 8, [255, 200, 50]);
    draw_circle(&mut canvas, 32, 34, 6, [255, 230, 100]);

    // Water/ground reflection
    for y in 40..64 {
        let mirror_y = 79 - y; // reflect sky
        for x in 0..64 {
            if mirror_y < 40 {
                let mut c = canvas[mirror_y][x];
                // Darken for water
                c[0] = (c[0] as f64 * 0.6) as u8;
                c[1] = (c[1] as f64 * 0.6) as u8;
                c[2] = (c[2] as f64 * 0.7) as u8;
                canvas[y][x] = c;
            }
        }
        // Shimmer
        scatter(&mut canvas, 0, y as i32, 64, y as i32 + 1, [255, 200, 100], 0.08, rng);
    }

    // Horizon line
    draw_rect(&mut canvas, 0, 39, 64, 41, [60, 30, 15]);

    canvas
}

fn generate_ocean(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let _sky = [135, 206, 250];
    let deep = palette.get(0).copied().unwrap_or([0, 50, 120]);
    let mid = palette.get(1).copied().unwrap_or([0, 105, 148]);
    let light = palette.get(2).copied().unwrap_or([30, 144, 255]);
    let foam = [220, 240, 255];

    // Sky
    for y in 0..25 {
        let t = y as f64 / 25.0;
        let r = (100.0 + 35.0 * t) as u8;
        let g = (180.0 + 26.0 * t) as u8;
        let b = (250.0) as u8;
        for x in 0..64 { canvas[y][x] = [r, g, b]; }
    }

    // Clouds
    for _ in 0..3 {
        let cx = rng.next_range(60) as i32;
        let cy = 5 + rng.next_range(12) as i32;
        draw_circle(&mut canvas, cx, cy, 4, [255, 255, 255]);
        draw_circle(&mut canvas, cx + 5, cy + 1, 3, [255, 255, 255]);
        draw_circle(&mut canvas, cx - 4, cy + 1, 3, [240, 240, 245]);
    }

    // Ocean with waves
    for y in 25..64 {
        let depth_t = (y - 25) as f64 / 39.0;
        let base = [
            (mid[0] as f64 * (1.0 - depth_t) + deep[0] as f64 * depth_t) as u8,
            (mid[1] as f64 * (1.0 - depth_t) + deep[1] as f64 * depth_t) as u8,
            (mid[2] as f64 * (1.0 - depth_t) + deep[2] as f64 * depth_t) as u8,
        ];
        for x in 0..64 {
            // Wave pattern
            let wave = ((x as f64 * 0.3 + y as f64 * 0.1 + rng.next_range(3) as f64).sin() * 0.5 + 0.5) > 0.7;
            canvas[y][x] = if wave { light } else { base };
        }
    }

    // Foam lines
    for wave_y in [26, 30, 36, 44] {
        let offset = rng.next_range(5) as i32;
        for x in (offset..62).step_by(3) {
            if rng.next_f64() > 0.3 {
                canvas[wave_y as usize][x as usize] = foam;
                if x + 1 < 64 { canvas[wave_y as usize][(x + 1) as usize] = foam; }
            }
        }
    }

    canvas
}

fn generate_city(_palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];

    // Night sky
    fill(&mut canvas, [15, 10, 40]);

    // Stars
    scatter(&mut canvas, 0, 0, 64, 35, [255, 255, 200], 0.03, rng);

    // Moon
    draw_circle(&mut canvas, 50, 10, 5, [240, 240, 200]);

    // Buildings
    let building_colors = [
        [40, 40, 60], [50, 50, 70], [35, 35, 55], [60, 60, 80],
    ];
    let window = [255, 230, 100];
    let window_off = [80, 80, 100];

    let mut x = 2;
    while x < 60 {
        let w = 5 + rng.next_range(8) as i32;
        let h = 15 + rng.next_range(25) as i32;
        let y_top = 62 - h;
        let bc = building_colors[rng.next_range(building_colors.len())];

        draw_rect(&mut canvas, x, y_top, x + w, 62, bc);

        // Windows
        let mut wy = y_top + 2;
        while wy < 60 {
            let mut wx = x + 1;
            while wx < x + w - 1 {
                let wc = if rng.next_f64() > 0.4 { window } else { window_off };
                if wx + 1 < 64 && wy + 1 < 64 {
                    canvas[wy as usize][wx as usize] = wc;
                    canvas[wy as usize][(wx + 1) as usize] = wc;
                    canvas[(wy + 1) as usize][wx as usize] = wc;
                    canvas[(wy + 1) as usize][(wx + 1) as usize] = wc;
                }
                wx += 3;
            }
            wy += 4;
        }

        x += w + 1 + rng.next_range(3) as i32;
    }

    // Ground
    draw_rect(&mut canvas, 0, 62, 64, 64, [30, 30, 30]);

    canvas
}

fn generate_heart(palette: &[[u8; 3]], _rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let bg = palette.get(palette.len().saturating_sub(1)).copied().unwrap_or([240, 220, 230]);
    let heart_color = palette.get(0).copied().unwrap_or([220, 20, 60]);
    let highlight = palette.get(1).copied().unwrap_or([255, 80, 100]);

    fill(&mut canvas, bg);

    // Heart shape using math
    for y in 0..64 {
        for x in 0..64 {
            let xn = (x as f64 - 32.0) / 16.0;
            let yn = (y as f64 - 34.0) / 16.0;
            let val = (xn * xn + yn * yn - 1.0).powi(3) - xn * xn * yn * yn * yn;
            if val <= 0.0 {
                canvas[y][x] = heart_color;
            }
        }
    }

    // Highlight
    draw_circle(&mut canvas, 24, 24, 4, highlight);

    canvas
}

fn generate_stars(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];

    // Dark space background with slight purple tint
    for y in 0..64 {
        for x in 0..64 {
            let noise = rng.next_range(15) as u8;
            canvas[y][x] = [5 + noise / 3, 2 + noise / 4, 15 + noise];
        }
    }

    // Nebula clouds
    let nebula_color = palette.get(0).copied().unwrap_or([100, 0, 150]);
    for _ in 0..5 {
        let cx = rng.next_range(64) as i32;
        let cy = rng.next_range(64) as i32;
        let r = 8 + rng.next_range(12) as i32;
        for y in (cy - r).max(0)..(cy + r).min(64) {
            for x in (cx - r).max(0)..(cx + r).min(64) {
                let dx = (x - cx) as f64;
                let dy = (y - cy) as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < r as f64 {
                    let t = 1.0 - dist / r as f64;
                    let alpha = t * t * 0.3;
                    let cur = canvas[y as usize][x as usize];
                    canvas[y as usize][x as usize] = [
                        (cur[0] as f64 * (1.0 - alpha) + nebula_color[0] as f64 * alpha) as u8,
                        (cur[1] as f64 * (1.0 - alpha) + nebula_color[1] as f64 * alpha) as u8,
                        (cur[2] as f64 * (1.0 - alpha) + nebula_color[2] as f64 * alpha) as u8,
                    ];
                }
            }
        }
    }

    // Stars
    for _ in 0..80 {
        let x = rng.next_range(64);
        let y = rng.next_range(64);
        let brightness = 180 + rng.next_range(75) as u8;
        canvas[y][x] = [brightness, brightness, brightness];
        // Some stars are bigger
        if rng.next_f64() > 0.7 {
            let c = [brightness, brightness, brightness - 20];
            if x + 1 < 64 { canvas[y][x + 1] = c; }
            if y + 1 < 64 { canvas[y + 1][x] = c; }
        }
    }

    // Bright colored stars
    for _ in 0..8 {
        let x = rng.next_range(62) + 1;
        let y = rng.next_range(62) + 1;
        let sc = palette.get(rng.next_range(palette.len())).copied().unwrap_or([255, 255, 100]);
        canvas[y][x] = sc;
        canvas[y - 1][x] = sc;
        canvas[y + 1][x] = sc;
        canvas[y][x - 1] = sc;
        canvas[y][x + 1] = sc;
    }

    canvas
}

fn generate_face(_palette: &[[u8; 3]], _rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let bg = [100, 180, 255];
    let face = [255, 220, 80];
    let eye_color = [50, 50, 50];

    fill(&mut canvas, bg);

    // Face circle
    draw_circle(&mut canvas, 32, 32, 22, face);

    // Eyes
    draw_circle(&mut canvas, 22, 26, 4, [255, 255, 255]);
    draw_circle(&mut canvas, 42, 26, 4, [255, 255, 255]);
    draw_circle(&mut canvas, 23, 26, 2, eye_color);
    draw_circle(&mut canvas, 43, 26, 2, eye_color);

    // Smile
    for x in 20..44 {
        let dx = (x as f64 - 32.0) / 12.0;
        let y = (38.0 + dx * dx * 4.0) as i32;
        if y >= 0 && y < 64 {
            canvas[y as usize][x] = eye_color;
            if y + 1 < 64 { canvas[(y + 1) as usize][x] = eye_color; }
        }
    }

    canvas
}

fn generate_flower(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let bg = [200, 230, 200];
    let stem = [34, 120, 34];
    let petal = palette.get(0).copied().unwrap_or([255, 100, 150]);
    let petal2 = palette.get(1).copied().unwrap_or([255, 60, 120]);
    let center = [255, 200, 50];

    fill(&mut canvas, bg);

    // Stem
    draw_rect(&mut canvas, 30, 35, 34, 60, stem);

    // Leaves
    draw_circle(&mut canvas, 26, 45, 4, [40, 160, 40]);
    draw_circle(&mut canvas, 38, 50, 4, [40, 160, 40]);

    // Petals
    let offsets = [(-10, -3), (10, -3), (-7, -10), (7, -10), (0, 8), (-10, 4), (10, 4)];
    for (dx, dy) in offsets {
        draw_circle(&mut canvas, 32 + dx, 25 + dy, 6, petal);
    }
    // Inner petals
    let inner_offsets = [(-6, -2), (6, -2), (0, -7), (0, 5)];
    for (dx, dy) in inner_offsets {
        draw_circle(&mut canvas, 32 + dx, 25 + dy, 4, petal2);
    }

    // Center
    draw_circle(&mut canvas, 32, 25, 5, center);

    // Ground
    draw_rect(&mut canvas, 0, 58, 64, 64, [100, 70, 40]);
    scatter(&mut canvas, 0, 55, 64, 58, [34, 139, 34], 0.4, rng);

    canvas
}

fn generate_sword(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];
    let bg = [30, 30, 50];
    let blade = [200, 200, 220];
    let blade_hi = [230, 230, 250];
    let guard = palette.get(0).copied().unwrap_or([180, 140, 50]);
    let grip = [80, 50, 30];
    let pommel = palette.get(1).copied().unwrap_or([200, 160, 60]);

    fill(&mut canvas, bg);

    // Blade (centered, vertical)
    draw_rect(&mut canvas, 29, 5, 35, 38, blade);
    draw_rect(&mut canvas, 30, 5, 32, 38, blade_hi);

    // Blade tip
    for y in 2..5 {
        let w = (5 - y) as i32;
        draw_rect(&mut canvas, 32 - w, y as i32, 32 + w, y as i32 + 1, blade);
    }

    // Guard (cross piece)
    draw_rect(&mut canvas, 22, 38, 42, 42, guard);

    // Grip
    draw_rect(&mut canvas, 29, 42, 35, 55, grip);
    // Wrap pattern
    for y in (43..55).step_by(3) {
        draw_rect(&mut canvas, 29, y, 35, y + 1, [100, 60, 35]);
    }

    // Pommel
    draw_circle(&mut canvas, 32, 57, 4, pommel);

    // Sparkle effects
    for _ in 0..6 {
        let sx = 26 + rng.next_range(12);
        let sy = 5 + rng.next_range(30);
        canvas[sy][sx] = [255, 255, 255];
    }

    canvas
}

fn generate_landscape(palette: &[[u8; 3]], rng: &mut Rng) -> Canvas {
    let mut canvas = vec![vec![[0u8; 3]; 64]; 64];

    // Sky gradient
    for y in 0..35 {
        let t = y as f64 / 35.0;
        let r = (100.0 + 80.0 * t) as u8;
        let g = (150.0 + 70.0 * t) as u8;
        let b = (255.0 - 30.0 * t) as u8;
        for x in 0..64 {
            canvas[y][x] = [r, g, b];
        }
    }

    // Clouds
    for _ in 0..2 {
        let cx = 10 + rng.next_range(44) as i32;
        let cy = 6 + rng.next_range(10) as i32;
        draw_circle(&mut canvas, cx, cy, 5, [255, 255, 255]);
        draw_circle(&mut canvas, cx + 6, cy + 1, 4, [245, 245, 250]);
        draw_circle(&mut canvas, cx - 5, cy + 1, 4, [245, 245, 250]);
    }

    // Rolling hills
    let c1 = palette.get(0).copied().unwrap_or([46, 139, 87]);
    let c2 = palette.get(1).copied().unwrap_or([34, 120, 34]);
    let c3 = palette.get(2).copied().unwrap_or([28, 100, 28]);

    // Far hills
    let offset1 = rng.next_range(10) as i32;
    for x in 0..64i32 {
        let h = 35 + ((x as f64 * 0.08 + offset1 as f64).sin() * 6.0) as i32;
        for y in h..50 {
            if y >= 0 && y < 64 { canvas[y as usize][x as usize] = c1; }
        }
    }

    // Near hills
    let offset2 = rng.next_range(10) as i32;
    for x in 0..64i32 {
        let h = 42 + ((x as f64 * 0.12 + offset2 as f64).sin() * 5.0) as i32;
        for y in h..64 {
            if y >= 0 && y < 64 { canvas[y as usize][x as usize] = c2; }
        }
    }

    // Foreground
    draw_rect(&mut canvas, 0, 54, 64, 64, c3);

    // Small trees on hills
    for _ in 0..4 {
        let tx = 5 + rng.next_range(54) as i32;
        let ty = 40 + rng.next_range(10) as i32;
        // Trunk
        draw_rect(&mut canvas, tx, ty, tx + 2, ty + 6, [80, 50, 30]);
        // Crown
        draw_circle(&mut canvas, tx + 1, ty - 2, 4, [30, 100, 30]);
    }

    // Flowers
    let flower_colors = [[255, 100, 100], [255, 200, 50], [200, 100, 255]];
    for _ in 0..8 {
        let fx = rng.next_range(64);
        let fy = 55 + rng.next_range(8);
        let fc = flower_colors[rng.next_range(3)];
        if fy < 64 { canvas[fy][fx] = fc; }
    }

    canvas
}
