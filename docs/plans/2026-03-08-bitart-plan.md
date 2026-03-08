# BitArt Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust TUI pixel art generator that creates 64x64 art from text prompts using Claude CLI or math fallback, renders in-terminal, and exports to PNG.

**Architecture:** Three modules — `generator` (AI + math pixel generation), `tui` (Ratatui fullscreen UI with input/canvas/status), `exporter` (PNG output via `image` crate). No async runtime; blocking generation in a spawned thread with channel signaling.

**Tech Stack:** Rust, ratatui, crossterm, image, serde_json

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/generator/mod.rs`
- Create: `src/generator/claude.rs`
- Create: `src/generator/fallback.rs`
- Create: `src/generator/palette.rs`
- Create: `src/tui/mod.rs`
- Create: `src/exporter/mod.rs`

**Step 1: Initialize Cargo project**

```bash
cd /Users/deathmac/Open\ Source/bitart
cargo init
```

**Step 2: Set up Cargo.toml with dependencies**

```toml
[package]
name = "bitart"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
image = "0.25"
serde_json = "1"
```

**Step 3: Create module files with empty stubs**

Create `src/generator/mod.rs`:
```rust
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
```

Create `src/generator/claude.rs`:
```rust
use super::Canvas;

pub fn generate(_prompt: &str) -> Result<Canvas, String> {
    Err("not implemented".into())
}
```

Create `src/generator/fallback.rs`:
```rust
use super::Canvas;

pub fn generate(_prompt: &str) -> Canvas {
    vec![vec![[0, 0, 0]; 64]; 64]
}
```

Create `src/generator/palette.rs`:
```rust
pub fn palette_for_prompt(_prompt: &str) -> Vec<[u8; 3]> {
    vec![[255, 255, 255]]
}
```

Create `src/tui/mod.rs`:
```rust
pub fn run() -> std::io::Result<()> {
    Ok(())
}
```

Create `src/exporter/mod.rs`:
```rust
use crate::generator::Canvas;

pub fn save_png(_canvas: &Canvas, _path: &str) -> Result<(), String> {
    Err("not implemented".into())
}
```

Create `src/main.rs`:
```rust
mod generator;
mod tui;
mod exporter;

fn main() -> std::io::Result<()> {
    tui::run()
}
```

**Step 4: Verify it compiles**

```bash
cargo build
```
Expected: Compiles with no errors.

**Step 5: Commit**

```bash
git init
git add Cargo.toml Cargo.lock src/
git commit -m "feat: scaffold bitart project with module stubs"
```

---

### Task 2: Palette System

**Files:**
- Modify: `src/generator/palette.rs`

**Step 1: Implement keyword-to-palette mapping**

Replace `src/generator/palette.rs`:
```rust
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
```

**Step 2: Verify it compiles**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/generator/palette.rs
git commit -m "feat: add keyword-based color palette system"
```

---

### Task 3: Math Fallback Generator

**Files:**
- Modify: `src/generator/fallback.rs`

**Step 1: Implement hash-seeded math generation**

Replace `src/generator/fallback.rs`:
```rust
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

    let s = seed as f64;
    let freq1 = ((seed % 7) as f64 + 1.0) * 0.15;
    let freq2 = ((seed % 11) as f64 + 1.0) * 0.1;
    let phase = (seed % 100) as f64 * 0.1;

    for y in 0..64 {
        for x in 0..64 {
            let xf = x as f64;
            let yf = y as f64;

            // Combine sine waves with XOR patterns
            let sine_val = (xf * freq1 + phase).sin() + (yf * freq2 + phase).cos();
            let xor_val = ((x ^ y) as f64 * 0.05 + s * 0.001).sin();
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
```

**Step 2: Verify it compiles**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/generator/fallback.rs
git commit -m "feat: add math-based fallback pixel art generator"
```

---

### Task 4: Claude CLI Generator

**Files:**
- Modify: `src/generator/claude.rs`

**Step 1: Implement Claude CLI integration**

Replace `src/generator/claude.rs`:
```rust
use super::Canvas;
use std::process::Command;
use std::time::Duration;

/// Attempt to generate pixel art by shelling out to the claude CLI.
pub fn generate(prompt: &str) -> Result<Canvas, String> {
    // Check if claude is in PATH
    let claude_prompt = format!(
        "You are a pixel art generator. Return ONLY a valid JSON array of 64 arrays, \
         each containing 64 hex color strings (#RRGGBB). No explanation, no markdown, \
         raw JSON only. Generate pixel art for: {}",
        prompt
    );

    let output = Command::new("claude")
        .arg("-p")
        .arg(&claude_prompt)
        .arg("--output-format")
        .arg("text")
        .output()
        .map_err(|e| format!("Failed to run claude: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude exited with error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_claude_output(&stdout)
}

/// Parse the JSON output from claude into a Canvas.
fn parse_claude_output(raw: &str) -> Result<Canvas, String> {
    // Find the JSON array in the output (skip any preamble text)
    let trimmed = raw.trim();
    let json_start = trimmed.find('[').ok_or("No JSON array found in output")?;
    let json_str = &trimmed[json_start..];

    let parsed: Vec<Vec<String>> =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    if parsed.len() != 64 {
        return Err(format!("Expected 64 rows, got {}", parsed.len()));
    }

    let mut canvas: Canvas = Vec::with_capacity(64);
    for (y, row) in parsed.iter().enumerate() {
        if row.len() != 64 {
            return Err(format!("Row {} has {} columns, expected 64", y, row.len()));
        }
        let mut canvas_row = Vec::with_capacity(64);
        for hex in row {
            let color = parse_hex_color(hex)
                .ok_or_else(|| format!("Invalid hex color '{}' at row {}", hex, y))?;
            canvas_row.push(color);
        }
        canvas.push(canvas_row);
    }

    Ok(canvas)
}

/// Parse a hex color string like "#FF00AA" into [r, g, b].
fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}
```

**Step 2: Verify it compiles**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/generator/claude.rs
git commit -m "feat: add Claude CLI pixel art generation"
```

---

### Task 5: Generator Orchestrator

**Files:**
- Modify: `src/generator/mod.rs`

**Step 1: Add seed-based regeneration and thread support**

Replace `src/generator/mod.rs`:
```rust
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
```

**Step 2: Verify it compiles**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/generator/mod.rs
git commit -m "feat: add generator orchestrator with async thread support"
```

---

### Task 6: PNG Exporter

**Files:**
- Modify: `src/exporter/mod.rs`

**Step 1: Implement PNG export with 8x scaling**

Replace `src/exporter/mod.rs`:
```rust
use crate::generator::Canvas;
use image::RgbImage;

/// Save the 64x64 canvas as a 512x512 PNG (8x scale).
pub fn save_png(canvas: &Canvas, path: &str) -> Result<(), String> {
    let scale: u32 = 8;
    let size = 64 * scale; // 512
    let mut img = RgbImage::new(size, size);

    for (y, row) in canvas.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            // Fill the scaled pixel block
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
```

**Step 2: Verify it compiles**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/exporter/mod.rs
git commit -m "feat: add PNG export with 8x scaling"
```

---

### Task 7: TUI — App State and Main Loop

**Files:**
- Modify: `src/tui/mod.rs`
- Modify: `src/main.rs`

**Step 1: Implement full TUI**

Replace `src/tui/mod.rs`:
```rust
use crate::exporter;
use crate::generator::{self, Canvas, GenerationMode, GenerationResult};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const TITLE_ART: &str = r#"
 ____  ___ _____  _    ____ _____
| __ )|_ _|_   _|/ \  |  _ \_   _|
|  _ \ | |  | | / _ \ | |_) || |
| |_) || |  | |/ ___ \|  _ < | |
|____/|___| |_/_/   \_\_| \_\|_|
"#;

enum AppState {
    Idle,
    Generating,
    Ready,
}

pub struct App {
    input: String,
    character_index: usize,
    state: AppState,
    canvas: Option<Canvas>,
    mode: Option<GenerationMode>,
    status_message: String,
    seed: u64,
    prompt: String,
    receiver: Option<mpsc::Receiver<GenerationResult>>,
    spinner_frame: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
            state: AppState::Idle,
            canvas: None,
            mode: None,
            status_message: String::from("Type a prompt and press Enter to generate pixel art"),
            seed: 0,
            prompt: String::new(),
            receiver: None,
            spinner_frame: 0,
            should_quit: false,
        }
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn enter_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.input.insert(idx, c);
        self.character_index += 1;
    }

    fn delete_char(&mut self) {
        if self.character_index > 0 {
            let current = self.character_index;
            let before: String = self.input.chars().take(current - 1).collect();
            let after: String = self.input.chars().skip(current).collect();
            self.input = format!("{}{}", before, after);
            self.character_index -= 1;
        }
    }

    fn start_generation(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        self.prompt = self.input.clone();
        self.seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.state = AppState::Generating;
        self.spinner_frame = 0;
        self.status_message = format!("Generating: {}...", self.prompt);
        self.receiver = Some(generator::generate_async(self.prompt.clone(), self.seed));
    }

    fn regenerate(&mut self) {
        if self.prompt.is_empty() {
            return;
        }
        self.seed = self.seed.wrapping_add(12345);
        self.state = AppState::Generating;
        self.spinner_frame = 0;
        self.status_message = format!("Regenerating: {}...", self.prompt);
        self.receiver = Some(generator::generate_async(self.prompt.clone(), self.seed));
    }

    fn save(&mut self) {
        if let Some(ref canvas) = self.canvas {
            match exporter::save_png(canvas, "output.png") {
                Ok(()) => self.status_message = "Saved to output.png!".into(),
                Err(e) => self.status_message = format!("Save failed: {}", e),
            }
        }
    }

    fn check_generation(&mut self) {
        if let Some(ref rx) = self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    let mode = result.mode;
                    self.canvas = Some(result.canvas);
                    self.mode = Some(mode);
                    self.state = AppState::Ready;
                    self.status_message = format!(
                        "Prompt: \"{}\" | Seed: {} | Mode: {} | [s]ave [r]egenerate [q]uit",
                        self.prompt, self.seed, mode
                    );
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_frame += 1;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state = AppState::Idle;
                    self.status_message = "Generation failed unexpectedly".into();
                    self.receiver = None;
                }
            }
        }
    }
}

pub fn run() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run_app(terminal);
    ratatui::restore();
    result
}

fn run_app(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        // Poll with short timeout for responsive spinner
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match &app.state {
                    AppState::Idle | AppState::Ready => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('s') => {
                            if matches!(app.state, AppState::Ready) {
                                app.save();
                            } else {
                                app.enter_char('s');
                            }
                        }
                        KeyCode::Char('r') => {
                            if matches!(app.state, AppState::Ready) {
                                app.regenerate();
                            } else {
                                app.enter_char('r');
                            }
                        }
                        KeyCode::Enter => {
                            app.start_generation();
                        }
                        KeyCode::Backspace => {
                            app.delete_char();
                        }
                        KeyCode::Char(c) => {
                            app.enter_char(c);
                        }
                        _ => {}
                    },
                    AppState::Generating => {
                        // Only allow quit during generation
                        if key.code == KeyCode::Char('q') {
                            app.should_quit = true;
                        }
                    }
                }
            }
        }

        app.check_generation();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(7),  // Title
        Constraint::Min(10),   // Canvas
        Constraint::Length(3), // Input
        Constraint::Length(1), // Status
    ])
    .split(area);

    // Title
    let title = Paragraph::new(TITLE_ART)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Canvas area
    let canvas_block = Block::default()
        .borders(Borders::ALL)
        .title(" Canvas ")
        .style(Style::default().fg(Color::DarkGray));
    let inner = canvas_block.inner(chunks[1]);
    frame.render_widget(canvas_block, chunks[1]);

    match &app.state {
        AppState::Generating => {
            let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let spinner = spinners[app.spinner_frame % spinners.len()];
            let text = format!("{} Generating pixel art...", spinner);
            let p = Paragraph::new(text)
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Center);
            // Center vertically
            let vert = Layout::vertical([
                Constraint::Percentage(45),
                Constraint::Length(1),
                Constraint::Percentage(45),
            ])
            .split(inner);
            frame.render_widget(p, vert[1]);
        }
        AppState::Ready => {
            if let Some(ref canvas) = app.canvas {
                render_canvas(frame, inner, canvas);
            }
        }
        AppState::Idle => {
            let text = "Enter a prompt below and press Enter to generate pixel art";
            let p = Paragraph::new(text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            let vert = Layout::vertical([
                Constraint::Percentage(45),
                Constraint::Length(1),
                Constraint::Percentage(45),
            ])
            .split(inner);
            frame.render_widget(p, vert[1]);
        }
    }

    // Input bar
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" Enter prompt: ")
        .style(Style::default().fg(Color::White));
    let input_inner = input_block.inner(chunks[2]);
    frame.render_widget(input_block, chunks[2]);

    let input_text = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White));
    frame.render_widget(input_text, input_inner);

    // Show cursor in input area when idle
    if matches!(app.state, AppState::Idle) {
        frame.set_cursor_position(Position::new(
            input_inner.x + app.character_index as u16,
            input_inner.y,
        ));
    }

    // Status bar
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::Green));
    frame.render_widget(status, chunks[3]);
}

/// Render the 64x64 canvas scaled to fit the available terminal area.
/// Each pixel uses "██" (two block chars) since terminal chars are ~2:1 height:width.
fn render_canvas(frame: &mut Frame, area: ratatui::layout::Rect, canvas: &Canvas) {
    let available_w = area.width as usize / 2; // each pixel = 2 chars wide
    let available_h = area.height as usize;

    // Scale to fit
    let scale_x = available_w as f64 / 64.0;
    let scale_y = available_h as f64 / 64.0;
    let scale = scale_x.min(scale_y).min(1.0); // don't upscale beyond 1:1

    let render_w = (64.0 * scale) as usize;
    let render_h = (64.0 * scale) as usize;

    // Center offset
    let offset_x = (area.width as usize - render_w * 2) / 2;
    let offset_y = (area.height as usize - render_h) / 2;

    let mut lines: Vec<Line> = Vec::new();

    // Pad top
    for _ in 0..offset_y {
        lines.push(Line::from(""));
    }

    for row in 0..render_h {
        let src_y = ((row as f64 / scale) as usize).min(63);
        let mut spans: Vec<Span> = Vec::new();

        // Pad left
        if offset_x > 0 {
            spans.push(Span::raw(" ".repeat(offset_x)));
        }

        for col in 0..render_w {
            let src_x = ((col as f64 / scale) as usize).min(63);
            let [r, g, b] = canvas[src_y][src_x];
            spans.push(Span::styled(
                "██",
                Style::default().fg(Color::Rgb(r, g, b)),
            ));
        }
        lines.push(Line::from(spans));
    }

    let canvas_widget = Paragraph::new(lines);
    frame.render_widget(canvas_widget, area);
}
```

**Step 2: Update `src/main.rs`** (should already be correct from Task 1, verify)

```rust
mod generator;
mod tui;
mod exporter;

fn main() -> std::io::Result<()> {
    tui::run()
}
```

**Step 3: Verify it compiles and runs**

```bash
cargo build
```

**Step 4: Manual test — run the app**

```bash
cargo run
```
Expected: TUI opens with title, empty canvas area, input bar, status bar. Type a prompt, press Enter, see spinner then pixel art. Press `s` to save, `r` to regenerate, `q` to quit.

**Step 5: Commit**

```bash
git add src/
git commit -m "feat: add full TUI with canvas rendering, input, and keybinds"
```

---

### Task 8: Integration Test and Polish

**Step 1: Verify full flow works**

```bash
cargo run
```

Test manually:
1. Type "neon city" → Enter → see generation → pixel art appears
2. Press `r` → regenerates with new seed
3. Press `s` → saves to output.png
4. Press `q` → exits cleanly

**Step 2: Verify PNG output**

```bash
file output.png
```
Expected: `output.png: PNG image data, 512 x 512`

**Step 3: Add .gitignore**

Create `.gitignore`:
```
/target
output.png
```

**Step 4: Commit**

```bash
git add .gitignore
git commit -m "chore: add .gitignore"
```
