use crate::exporter;
use crate::generator::{self, Canvas, GenerationMode, GenerationResult};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use std::sync::mpsc;
use std::time::Duration;

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

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match &app.state {
                    AppState::Idle => match key.code {
                        KeyCode::Char('q') if app.input.is_empty() => {
                            app.should_quit = true;
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
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        _ => {}
                    },
                    AppState::Ready => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('s') => {
                            app.save();
                        }
                        KeyCode::Char('r') => {
                            app.regenerate();
                        }
                        KeyCode::Enter => {
                            // Go back to idle to type new prompt
                            app.state = AppState::Idle;
                            app.input.clear();
                            app.character_index = 0;
                            app.status_message = "Type a new prompt and press Enter".into();
                        }
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        _ => {}
                    },
                    AppState::Generating => {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
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
            let spinners = ['|', '/', '-', '\\'];
            let spinner = spinners[app.spinner_frame % spinners.len()];
            let text = format!("{} Generating pixel art...", spinner);
            let p = Paragraph::new(text)
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Center);
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
    let input_title = if matches!(app.state, AppState::Ready) {
        " Press Enter for new prompt: "
    } else {
        " Enter prompt: "
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(input_title)
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
fn render_canvas(frame: &mut Frame, area: ratatui::layout::Rect, canvas: &Canvas) {
    let available_w = area.width as usize / 2; // each pixel = 2 chars wide
    let available_h = area.height as usize;

    let scale_x = available_w as f64 / 64.0;
    let scale_y = available_h as f64 / 64.0;
    let scale = scale_x.min(scale_y).min(1.0);

    let render_w = (64.0 * scale) as usize;
    let render_h = (64.0 * scale) as usize;

    let offset_x = (area.width as usize - render_w * 2) / 2;
    let offset_y = (area.height as usize - render_h) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for _ in 0..offset_y {
        lines.push(Line::from(""));
    }

    for row in 0..render_h {
        let src_y = ((row as f64 / scale) as usize).min(63);
        let mut spans: Vec<Span> = Vec::new();

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
