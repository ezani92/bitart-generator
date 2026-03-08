use crate::config::Config;
use crate::exporter;
use crate::generator::{self, Canvas, GenerationResult};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const TITLE_ART: &str = r#"
 ____  ___ _____  _    ____ _____
| __ )|_ _|_   _|/ \  |  _ \_   _|
|  _ \ | |  | | / _ \ | |_) || |
| |_) || |  | |/ ___ \|  _ < | |
|____/|___| |_/_/   \_\_| \_\|_|
"#;

enum Screen {
    Setup(SetupState),
    Main(AppState),
}

struct SetupState {
    step: SetupStep,
    selected_model: usize,
    api_key_input: String,
    cursor_index: usize,
    error_message: Option<String>,
}

enum SetupStep {
    SelectModel,
    EnterApiKey,
}

enum AppState {
    Idle,
    Generating,
    Ready,
}

pub struct App {
    screen: Screen,
    config: Option<Config>,
    input: String,
    character_index: usize,
    canvas: Option<Canvas>,
    model_name: Option<String>,
    status_message: String,
    prompt: String,
    receiver: Option<mpsc::Receiver<Result<GenerationResult, String>>>,
    generation_start: Option<Instant>,
    spinner_frame: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        let screen = if config.is_some() {
            Screen::Main(AppState::Idle)
        } else {
            Screen::Setup(SetupState {
                step: SetupStep::SelectModel,
                selected_model: 0,
                api_key_input: String::new(),
                cursor_index: 0,
                error_message: None,
            })
        };

        Self {
            screen,
            config,
            input: String::new(),
            character_index: 0,
            canvas: None,
            model_name: None,
            status_message: String::from("Type a prompt and press Enter to generate pixel art"),
            prompt: String::new(),
            receiver: None,
            generation_start: None,
            spinner_frame: 0,
            should_quit: false,
        }
    }

    fn byte_index(input: &str, char_index: usize) -> usize {
        input
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_index)
            .unwrap_or(input.len())
    }

    fn start_generation(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        if let Some(ref config) = self.config {
            self.prompt = self.input.clone();
            self.screen = Screen::Main(AppState::Generating);
            self.spinner_frame = 0;
            self.generation_start = Some(Instant::now());
            self.status_message = format!("Generating: {}...", self.prompt);
            self.receiver = Some(generator::generate_async(
                self.prompt.clone(),
                config.api_key.clone(),
                config.model.clone(),
            ));
        }
    }

    fn regenerate(&mut self) {
        if self.prompt.is_empty() {
            return;
        }
        if let Some(ref config) = self.config {
            self.screen = Screen::Main(AppState::Generating);
            self.spinner_frame = 0;
            self.generation_start = Some(Instant::now());
            self.status_message = format!("Regenerating: {}...", self.prompt);
            self.receiver = Some(generator::generate_async(
                self.prompt.clone(),
                config.api_key.clone(),
                config.model.clone(),
            ));
        }
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
                    match result {
                        Ok(gen_result) => {
                            let model = gen_result.model.clone();
                            self.canvas = Some(gen_result.canvas);
                            self.model_name = Some(model.clone());
                            self.screen = Screen::Main(AppState::Ready);
                            self.generation_start = None;
                            self.status_message = format!(
                                "Prompt: \"{}\" | 64x64 | Model: {} | [s]ave [r]egenerate [q]uit",
                                self.prompt, model
                            );
                        }
                        Err(e) => {
                            self.screen = Screen::Main(AppState::Idle);
                            self.generation_start = None;
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_frame += 1;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.screen = Screen::Main(AppState::Idle);
                    self.generation_start = None;
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
                match &app.screen {
                    Screen::Setup(_) => handle_setup_input(&mut app, key.code),
                    Screen::Main(state) => match state {
                        AppState::Idle => match key.code {
                            KeyCode::Char('q') if app.input.is_empty() => {
                                app.should_quit = true;
                            }
                            KeyCode::Enter => {
                                app.start_generation();
                            }
                            KeyCode::Backspace => {
                                if app.character_index > 0 {
                                    let current = app.character_index;
                                    let before: String = app.input.chars().take(current - 1).collect();
                                    let after: String = app.input.chars().skip(current).collect();
                                    app.input = format!("{}{}", before, after);
                                    app.character_index -= 1;
                                }
                            }
                            KeyCode::Char(c) => {
                                let idx = App::byte_index(&app.input, app.character_index);
                                app.input.insert(idx, c);
                                app.character_index += 1;
                            }
                            KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            _ => {}
                        },
                        AppState::Ready => match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Char('s') => app.save(),
                            KeyCode::Char('r') => app.regenerate(),
                            KeyCode::Enter => {
                                app.screen = Screen::Main(AppState::Idle);
                                app.input.clear();
                                app.character_index = 0;
                                app.status_message = "Type a new prompt and press Enter".into();
                            }
                            KeyCode::Esc => app.should_quit = true,
                            _ => {}
                        },
                        AppState::Generating => {
                            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                                app.should_quit = true;
                            }
                        }
                    },
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

fn handle_setup_input(app: &mut App, key: KeyCode) {
    // We need to take ownership temporarily
    if let Screen::Setup(ref mut setup) = app.screen {
        match setup.step {
            SetupStep::SelectModel => match key {
                KeyCode::Up => {
                    if setup.selected_model > 0 {
                        setup.selected_model -= 1;
                    }
                }
                KeyCode::Down => {
                    let max = Config::available_models().len() - 1;
                    if setup.selected_model < max {
                        setup.selected_model += 1;
                    }
                }
                KeyCode::Enter => {
                    setup.step = SetupStep::EnterApiKey;
                }
                KeyCode::Esc => {
                    app.should_quit = true;
                }
                _ => {}
            },
            SetupStep::EnterApiKey => match key {
                KeyCode::Char(c) => {
                    let idx = App::byte_index(&setup.api_key_input, setup.cursor_index);
                    setup.api_key_input.insert(idx, c);
                    setup.cursor_index += 1;
                }
                KeyCode::Backspace => {
                    if setup.cursor_index > 0 {
                        let current = setup.cursor_index;
                        let before: String = setup.api_key_input.chars().take(current - 1).collect();
                        let after: String = setup.api_key_input.chars().skip(current).collect();
                        setup.api_key_input = format!("{}{}", before, after);
                        setup.cursor_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    if setup.api_key_input.trim().is_empty() {
                        setup.error_message = Some("API key cannot be empty".into());
                        return;
                    }
                    let models = Config::available_models();
                    let model = models[setup.selected_model].0.to_string();
                    let config = Config {
                        api_key: setup.api_key_input.clone(),
                        model,
                    };
                    match config.save() {
                        Ok(()) => {
                            app.config = Some(config);
                            app.screen = Screen::Main(AppState::Idle);
                        }
                        Err(e) => {
                            setup.error_message = Some(format!("Failed to save: {}", e));
                        }
                    }
                }
                KeyCode::Esc => {
                    setup.step = SetupStep::SelectModel;
                }
                _ => {}
            },
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::Setup(setup) => draw_setup(frame, setup),
        Screen::Main(_) => draw_main(frame, app),
    }
}

fn draw_setup(frame: &mut Frame, setup: &SetupState) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(8),  // Title
        Constraint::Min(10),   // Setup content
        Constraint::Length(2), // Help
    ])
    .split(area);

    // Title
    let title_with_version = format!("{}\n                          v{}", TITLE_ART.trim_end(), VERSION);
    let title = Paragraph::new(title_with_version)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    match setup.step {
        SetupStep::SelectModel => {
            let models = Config::available_models();
            let mut lines: Vec<Line> = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Welcome! Select an AI model for image generation:",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (i, (_, name, price)) in models.iter().enumerate() {
                let marker = if i == setup.selected_model { " > " } else { "   " };
                let style = if i == setup.selected_model {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}  —  {}", marker, name, price),
                    style,
                )));
            }

            if let Some(ref err) = setup.error_message {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("  Error: {}", err),
                    Style::default().fg(Color::Red),
                )));
            }

            let content = Paragraph::new(lines);
            frame.render_widget(content, chunks[1]);

            let help = Paragraph::new(" Up/Down: select | Enter: confirm | Esc: quit")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(help, chunks[2]);
        }
        SetupStep::EnterApiKey => {
            let models = Config::available_models();
            let selected = models[setup.selected_model];
            let mut lines: Vec<Line> = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  Selected model: {} ({})", selected.1, selected.2),
                    Style::default().fg(Color::Green),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Enter your OpenAI API key:",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "  Get one at: https://platform.openai.com/api-keys",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ];

            // Show masked API key
            let masked: String = if setup.api_key_input.is_empty() {
                String::new()
            } else if setup.api_key_input.len() <= 8 {
                "*".repeat(setup.api_key_input.len())
            } else {
                let visible = &setup.api_key_input[..4];
                format!("{}{}", visible, "*".repeat(setup.api_key_input.len() - 4))
            };
            lines.push(Line::from(vec![
                Span::styled("  > ", Style::default().fg(Color::Yellow)),
                Span::styled(masked, Style::default().fg(Color::White)),
            ]));

            if let Some(ref err) = setup.error_message {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("  {}", err),
                    Style::default().fg(Color::Red),
                )));
            }

            let content = Paragraph::new(lines);
            frame.render_widget(content, chunks[1]);

            let help = Paragraph::new(" Enter: confirm | Esc: back")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(help, chunks[2]);
        }
    }
}

fn draw_main(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(8),  // Title + version
        Constraint::Min(10),   // Canvas
        Constraint::Length(3), // Input
        Constraint::Length(1), // Status
    ])
    .split(area);

    // Title with version below
    let title_with_version = format!("{}\n                          v{}", TITLE_ART.trim_end(), VERSION);
    let title = Paragraph::new(title_with_version)
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

    match &app.screen {
        Screen::Main(AppState::Generating) => {
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
        Screen::Main(AppState::Ready) => {
            if let Some(ref canvas) = app.canvas {
                render_canvas(frame, inner, canvas);
            }
        }
        _ => {
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
    let input_title = if matches!(app.screen, Screen::Main(AppState::Ready)) {
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

    if matches!(app.screen, Screen::Main(AppState::Idle)) {
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
    let available_w = area.width as usize / 2;
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
