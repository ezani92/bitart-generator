use crate::config::Config;
use crate::exporter;
use crate::generator::{self, Canvas, ChainResult, FramesResult, GenerationResult};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUOTES: &[&str] = &[
    "Every pixel tells a story.",
    "Creativity is intelligence having fun. — Albert Einstein",
    "Art is not what you see, but what you make others see. — Edgar Degas",
    "The only way to do great work is to love what you do. — Steve Jobs",
    "Simplicity is the ultimate sophistication. — Leonardo da Vinci",
    "Imagination is the beginning of creation. — George Bernard Shaw",
    "A picture is worth a thousand pixels.",
    "The best time to create was yesterday. The next best time is now.",
    "Art enables us to find ourselves and lose ourselves at the same time. — Thomas Merton",
    "Every artist was first an amateur. — Ralph Waldo Emerson",
    "Life is short. Make every pixel count.",
    "Color is a power which directly influences the soul. — Wassily Kandinsky",
    "To create is to live twice. — Albert Camus",
    "The earth has music for those who listen. — Shakespeare",
    "In the middle of difficulty lies opportunity. — Albert Einstein",
    "Stay hungry, stay foolish. — Steve Jobs",
    "The purpose of art is washing the dust of daily life off our souls. — Pablo Picasso",
    "Dream big, start small, act now.",
    "Everything you can imagine is real. — Pablo Picasso",
    "Be yourself; everyone else is already taken. — Oscar Wilde",
];

const TITLE_ART: &str = "
░██        ░██   ░██                           ░██
░██              ░██                           ░██
░████████  ░██░████████  ░██████   ░██░████ ░████████
░██    ░██ ░██   ░██          ░██  ░███        ░██
░██    ░██ ░██   ░██     ░███████  ░██         ░██
░███   ░██ ░██   ░██    ░██   ░██  ░██         ░██
░██░█████  ░██    ░████  ░█████░██ ░██          ░████
";

#[derive(Clone, Copy, PartialEq)]
pub enum ExportMode {
    Png,
    Gif,
    ChainPng,
}

impl ExportMode {
    fn label(&self) -> &str {
        match self {
            ExportMode::Png => "PNG",
            ExportMode::Gif => "GIF",
            ExportMode::ChainPng => "CHAIN",
        }
    }

    fn toggle(&self) -> Self {
        match self {
            ExportMode::Png => ExportMode::Gif,
            ExportMode::Gif => ExportMode::ChainPng,
            ExportMode::ChainPng => ExportMode::Png,
        }
    }
}

enum Screen {
    Setup(SetupState),
    Main(AppState),
}

struct SetupState {
    step: SetupStep,
    menu_selection: usize,
    selected_model: usize,
    api_key_input: String,
    output_dir_input: String,
    cursor_index: usize,
    error_message: Option<String>,
    is_reconfigure: bool,
}

enum SetupStep {
    ConfigMenu,
    SelectModel,
    EnterApiKey,
    SetOutputDir,
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
    export_mode: ExportMode,
    canvas: Option<Canvas>,
    frames: Option<Vec<Canvas>>,
    model_name: Option<String>,
    status_message: String,
    prompt: String,
    receiver: Option<mpsc::Receiver<Result<GenerationResult, String>>>,
    frames_receiver: Option<mpsc::Receiver<Result<FramesResult, String>>>,
    chain_receiver: Option<mpsc::Receiver<Result<ChainResult, String>>>,
    chain_tiles: Option<Vec<(String, Canvas)>>,
    chain_tile_index: usize,
    generation_start: Option<Instant>,
    spinner_frame: usize,
    gif_frame_index: usize,
    gif_last_tick: Option<Instant>,
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
                menu_selection: 0,
                selected_model: 0,
                api_key_input: String::new(),
                output_dir_input: Config::default_output_dir(),
                cursor_index: 0,
                error_message: None,
                is_reconfigure: false,
            })
        };

        Self {
            screen,
            config,
            input: String::new(),
            character_index: 0,
            export_mode: ExportMode::Png,
            canvas: None,
            frames: None,
            model_name: None,
            status_message: String::from("Type a prompt and press Enter to generate | Ctrl+[c]onfig Ctrl+[q]uit"),
            prompt: String::new(),
            receiver: None,
            frames_receiver: None,
            chain_receiver: None,
            chain_tiles: None,
            chain_tile_index: 0,
            generation_start: None,
            spinner_frame: 0,
            gif_frame_index: 0,
            gif_last_tick: None,
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

    fn mode_indicator(&self) -> String {
        format!("[Shift+Tab: {}]", self.export_mode.label())
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

            match self.export_mode {
                ExportMode::Png => {
                    self.status_message = format!("Generating: {}...", self.prompt);
                    self.receiver = Some(generator::generate_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
                ExportMode::Gif => {
                    self.status_message = format!("Generating 3 frames: {}...", self.prompt);
                    self.frames_receiver = Some(generator::generate_frames_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
                ExportMode::ChainPng => {
                    self.status_message = format!("Analyzing & generating tiles: {}...", self.prompt);
                    self.chain_receiver = Some(generator::generate_chain_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
            }
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

            match self.export_mode {
                ExportMode::Png => {
                    self.status_message = format!("Regenerating: {}...", self.prompt);
                    self.receiver = Some(generator::generate_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
                ExportMode::Gif => {
                    self.status_message = format!("Regenerating 3 frames: {}...", self.prompt);
                    self.frames_receiver = Some(generator::generate_frames_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
                ExportMode::ChainPng => {
                    self.status_message = format!("Regenerating tiles: {}...", self.prompt);
                    self.chain_receiver = Some(generator::generate_chain_async(
                        self.prompt.clone(),
                        config.api_key.clone(),
                        config.model.clone(),
                    ));
                }
            }
        }
    }

    fn save(&mut self) {
        match self.export_mode {
            ExportMode::ChainPng => {
                if let Some(ref tiles) = self.chain_tiles {
                    let base_dir = if let Some(ref config) = self.config {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let folder = format!("bitart_chain_{}", timestamp);
                        match &config.output_dir {
                            Some(dir) => {
                                std::path::Path::new(dir)
                                    .join(&folder)
                                    .to_string_lossy()
                                    .to_string()
                            }
                            None => folder,
                        }
                    } else {
                        "chain_output".to_string()
                    };

                    let tile_refs: Vec<(String, &Canvas)> = tiles
                        .iter()
                        .map(|(name, canvas)| (name.clone(), canvas))
                        .collect();

                    match exporter::save_chain_pngs(&tile_refs, &base_dir) {
                        Ok(paths) => {
                            self.status_message = format!(
                                "Saved {} tiles to {}!",
                                paths.len(),
                                base_dir
                            );
                        }
                        Err(e) => self.status_message = format!("Save failed: {}", e),
                    }
                }
            }
            _ => {
                let path = if let Some(ref config) = self.config {
                    match self.export_mode {
                        ExportMode::Png => config.output_path("png"),
                        ExportMode::Gif => config.output_path("gif"),
                        ExportMode::ChainPng => unreachable!(),
                    }
                } else {
                    match self.export_mode {
                        ExportMode::Png => "output.png".to_string(),
                        ExportMode::Gif => "output.gif".to_string(),
                        ExportMode::ChainPng => unreachable!(),
                    }
                };

                match self.export_mode {
                    ExportMode::Png => {
                        if let Some(ref canvas) = self.canvas {
                            match exporter::save_png(canvas, &path) {
                                Ok(()) => self.status_message = format!("Saved to {}!", path),
                                Err(e) => self.status_message = format!("Save failed: {}", e),
                            }
                        }
                    }
                    ExportMode::Gif => {
                        if let Some(ref frames) = self.frames {
                            match exporter::save_gif(frames, &path) {
                                Ok(()) => self.status_message = format!("Saved to {}!", path),
                                Err(e) => self.status_message = format!("Save failed: {}", e),
                            }
                        }
                    }
                    ExportMode::ChainPng => unreachable!(),
                }
            }
        }
    }

    fn open_config(&mut self) {
        let models = Config::available_models();
        let (selected_model, api_key_input, output_dir_input) = if let Some(ref config) = self.config {
            let idx = models.iter().position(|(id, _, _)| *id == config.model).unwrap_or(0);
            let dir = config.output_dir.clone().unwrap_or_else(Config::default_output_dir);
            (idx, config.api_key.clone(), dir)
        } else {
            (0, String::new(), Config::default_output_dir())
        };
        let cursor_index = api_key_input.chars().count();
        self.screen = Screen::Setup(SetupState {
            step: SetupStep::ConfigMenu,
            menu_selection: 0,
            selected_model,
            api_key_input,
            output_dir_input,
            cursor_index,
            error_message: None,
            is_reconfigure: true,
        });
    }

    fn ready_status(&self) -> String {
        let model = self.model_name.as_deref().unwrap_or("unknown");
        if self.export_mode == ExportMode::ChainPng {
            if let Some(ref tiles) = self.chain_tiles {
                let total = tiles.len();
                let current = self.chain_tile_index + 1;
                let tile_name = &tiles[self.chain_tile_index].0;
                return format!(
                    "\"{}\" | Tile {}/{}: {} | {} | Left/Right: browse | Ctrl+[n]ew Ctrl+[s]ave Ctrl+[r]egenerate Ctrl+[c]onfig Ctrl+[q]uit",
                    self.prompt, current, total, tile_name, model
                );
            }
        }
        let ext = if self.export_mode == ExportMode::Gif { "GIF 3fps" } else { "PNG" };
        let res = if let Some(ref c) = self.canvas {
            let h = c.len();
            let w = if h > 0 { c[0].len() } else { 0 };
            format!("{}x{}", w, h)
        } else {
            "—".to_string()
        };
        format!(
            "\"{}\" | {} {} | {} | Ctrl+[n]ew Ctrl+[s]ave Ctrl+[r]egenerate Ctrl+[c]onfig Ctrl+[q]uit",
            self.prompt, res, ext, model
        )
    }

    fn check_generation(&mut self) {
        // Check single-frame (PNG mode)
        if let Some(ref rx) = self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(gen_result) => {
                            self.model_name = Some(gen_result.model.clone());
                            self.canvas = Some(gen_result.canvas);
                            self.frames = None;
                            self.screen = Screen::Main(AppState::Ready);
                            self.generation_start = None;
                            self.status_message = self.ready_status();
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

        // Check chain mode
        if let Some(ref rx) = self.chain_receiver {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(chain_result) => {
                            self.model_name = Some(chain_result.model.clone());
                            let tiles: Vec<(String, Canvas)> = chain_result
                                .tiles
                                .into_iter()
                                .map(|t| (t.name, t.canvas))
                                .collect();
                            // Show first tile
                            self.canvas = Some(tiles[0].1.clone());
                            self.chain_tile_index = 0;
                            self.chain_tiles = Some(tiles);
                            self.frames = None;
                            self.screen = Screen::Main(AppState::Ready);
                            self.generation_start = None;
                            self.status_message = self.ready_status();
                        }
                        Err(e) => {
                            self.screen = Screen::Main(AppState::Idle);
                            self.generation_start = None;
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                    self.chain_receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_frame += 1;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.screen = Screen::Main(AppState::Idle);
                    self.generation_start = None;
                    self.status_message = "Generation failed unexpectedly".into();
                    self.chain_receiver = None;
                }
            }
        }

        // Check multi-frame (GIF mode)
        if let Some(ref rx) = self.frames_receiver {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(frames_result) => {
                            self.model_name = Some(frames_result.model.clone());
                            self.canvas = Some(frames_result.frames[0].clone());
                            self.frames = Some(frames_result.frames);
                            self.gif_frame_index = 0;
                            self.gif_last_tick = Some(Instant::now());
                            self.screen = Screen::Main(AppState::Ready);
                            self.generation_start = None;
                            self.status_message = self.ready_status();
                        }
                        Err(e) => {
                            self.screen = Screen::Main(AppState::Idle);
                            self.generation_start = None;
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                    self.frames_receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_frame += 1;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.screen = Screen::Main(AppState::Idle);
                    self.generation_start = None;
                    self.status_message = "Generation failed unexpectedly".into();
                    self.frames_receiver = None;
                }
            }
        }
    }

    fn tick_gif_animation(&mut self) {
        if let Some(ref frames) = self.frames {
            if let Some(last) = self.gif_last_tick {
                if last.elapsed() >= Duration::from_millis(333) {
                    self.gif_frame_index = (self.gif_frame_index + 1) % frames.len();
                    self.canvas = Some(frames[self.gif_frame_index].clone());
                    self.gif_last_tick = Some(Instant::now());
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
                // Shift+Tab toggles mode in Idle and Ready states
                if key.code == KeyCode::BackTab {
                    if matches!(app.screen, Screen::Main(AppState::Idle) | Screen::Main(AppState::Ready)) {
                        app.export_mode = app.export_mode.toggle();
                        if matches!(app.screen, Screen::Main(AppState::Ready)) {
                            app.status_message = app.ready_status();
                        } else {
                            app.status_message = format!(
                                "Mode: {} | Type a prompt and press Enter to generate | Ctrl+[c]onfig Ctrl+[q]uit",
                                app.export_mode.label()
                            );
                        }
                    }
                    continue;
                }

                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                match &app.screen {
                    Screen::Setup(_) => handle_setup_input(&mut app, key.code),
                    Screen::Main(state) => match state {
                        AppState::Idle => {
                            if ctrl {
                                match key.code {
                                    KeyCode::Char('c') => app.open_config(),
                                    KeyCode::Char('q') => app.should_quit = true,
                                    _ => {}
                                }
                            } else {
                                match key.code {
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
                                }
                            }
                        },
                        AppState::Ready => {
                            if ctrl {
                                match key.code {
                                    KeyCode::Char('q') => app.should_quit = true,
                                    KeyCode::Char('c') => app.open_config(),
                                    KeyCode::Char('s') => app.save(),
                                    KeyCode::Char('r') => app.regenerate(),
                                    KeyCode::Char('n') => {
                                        app.screen = Screen::Main(AppState::Idle);
                                        app.input.clear();
                                        app.character_index = 0;
                                        app.status_message = "Type a prompt and press Enter to generate | Ctrl+[c]onfig Ctrl+[q]uit".into();
                                    }
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Enter => {
                                        app.screen = Screen::Main(AppState::Idle);
                                        app.input.clear();
                                        app.character_index = 0;
                                        app.status_message = "Type a prompt and press Enter to generate | Ctrl+[c]onfig Ctrl+[q]uit".into();
                                    }
                                    KeyCode::Left => {
                                        if app.export_mode == ExportMode::ChainPng {
                                            if let Some(ref tiles) = app.chain_tiles {
                                                if app.chain_tile_index > 0 {
                                                    app.chain_tile_index -= 1;
                                                } else {
                                                    app.chain_tile_index = tiles.len() - 1;
                                                }
                                                app.canvas = Some(tiles[app.chain_tile_index].1.clone());
                                                app.status_message = app.ready_status();
                                            }
                                        }
                                    }
                                    KeyCode::Right => {
                                        if app.export_mode == ExportMode::ChainPng {
                                            if let Some(ref tiles) = app.chain_tiles {
                                                if app.chain_tile_index < tiles.len() - 1 {
                                                    app.chain_tile_index += 1;
                                                } else {
                                                    app.chain_tile_index = 0;
                                                }
                                                app.canvas = Some(tiles[app.chain_tile_index].1.clone());
                                                app.status_message = app.ready_status();
                                            }
                                        }
                                    }
                                    KeyCode::Esc => app.should_quit = true,
                                    _ => {}
                                }
                            }
                        },
                        AppState::Generating => {
                            if (ctrl && key.code == KeyCode::Char('q')) || key.code == KeyCode::Esc {
                                app.should_quit = true;
                            }
                        }
                    },
                }
            }
        }

        app.check_generation();
        app.tick_gif_animation();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_setup_input(app: &mut App, key: KeyCode) {
    if let Screen::Setup(ref mut setup) = app.screen {
        match setup.step {
            SetupStep::ConfigMenu => match key {
                KeyCode::Up => {
                    if setup.menu_selection > 0 {
                        setup.menu_selection -= 1;
                    }
                }
                KeyCode::Down => {
                    if setup.menu_selection < 1 {
                        setup.menu_selection += 1;
                    }
                }
                KeyCode::Enter => {
                    match setup.menu_selection {
                        0 => {
                            // Update Model & API Key
                            setup.step = SetupStep::SelectModel;
                        }
                        1 => {
                            // Update Save Folder
                            setup.cursor_index = setup.output_dir_input.chars().count();
                            setup.step = SetupStep::SetOutputDir;
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    app.screen = Screen::Main(AppState::Idle);
                }
                _ => {}
            },
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
                    if setup.is_reconfigure {
                        setup.step = SetupStep::ConfigMenu;
                    } else {
                        app.should_quit = true;
                    }
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
                    setup.error_message = None;
                    setup.cursor_index = setup.output_dir_input.chars().count();
                    setup.step = SetupStep::SetOutputDir;
                }
                KeyCode::Esc => {
                    setup.step = SetupStep::SelectModel;
                }
                _ => {}
            },
            SetupStep::SetOutputDir => match key {
                KeyCode::Char(c) => {
                    let idx = App::byte_index(&setup.output_dir_input, setup.cursor_index);
                    setup.output_dir_input.insert(idx, c);
                    setup.cursor_index += 1;
                }
                KeyCode::Backspace => {
                    if setup.cursor_index > 0 {
                        let current = setup.cursor_index;
                        let before: String = setup.output_dir_input.chars().take(current - 1).collect();
                        let after: String = setup.output_dir_input.chars().skip(current).collect();
                        setup.output_dir_input = format!("{}{}", before, after);
                        setup.cursor_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    let output_dir = if setup.output_dir_input.trim().is_empty() {
                        None
                    } else {
                        Some(setup.output_dir_input.clone())
                    };
                    let models = Config::available_models();
                    let model = models[setup.selected_model].0.to_string();
                    let config = Config {
                        api_key: setup.api_key_input.clone(),
                        model,
                        output_dir,
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
                    if setup.is_reconfigure {
                        setup.step = SetupStep::ConfigMenu;
                    } else {
                        setup.step = SetupStep::EnterApiKey;
                        setup.cursor_index = setup.api_key_input.chars().count();
                    }
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
        Constraint::Length(10), // Title
        Constraint::Min(10),   // Setup content
        Constraint::Length(2), // Help
    ])
    .split(area);

    // Title
    let title_with_version = format!("{}\n v{}", TITLE_ART.trim_end(), VERSION);
    let title = Paragraph::new(title_with_version)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    match setup.step {
        SetupStep::ConfigMenu => {
            let menu_items = [
                "Update Model & API Key",
                "Update Save Folder",
            ];
            let mut lines: Vec<Line> = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Settings:",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (i, item) in menu_items.iter().enumerate() {
                let marker = if i == setup.menu_selection { " > " } else { "   " };
                let style = if i == setup.menu_selection {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}. {}", marker, i + 1, item),
                    style,
                )));
            }

            let content = Paragraph::new(lines);
            frame.render_widget(content, chunks[1]);

            let help = Paragraph::new(" Up/Down: select | Enter: confirm | Esc: back")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(help, chunks[2]);
        }
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

            let help_text = if setup.is_reconfigure {
                " Up/Down: select | Enter: confirm | Esc: cancel"
            } else {
                " Up/Down: select | Enter: confirm | Esc: quit"
            };
            let help = Paragraph::new(help_text)
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
        SetupStep::SetOutputDir => {
            let mut lines: Vec<Line> = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Set default output folder:",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "  Exported PNG/GIF files will be saved here. Leave empty to save in current directory.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ];

            lines.push(Line::from(vec![
                Span::styled("  > ", Style::default().fg(Color::Yellow)),
                Span::styled(&setup.output_dir_input, Style::default().fg(Color::White)),
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
        Constraint::Length(10), // Title + version
        Constraint::Min(10),   // Canvas
        Constraint::Length(3), // Input
        Constraint::Length(1), // Status
    ])
    .split(area);

    // Title with version and mode indicator
    let title_with_version = format!(
        "{}\n v{}  {}",
        TITLE_ART.trim_end(),
        VERSION,
        app.mode_indicator()
    );
    let title = Paragraph::new(title_with_version)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
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
            let gen_text = match app.export_mode {
                ExportMode::Gif => format!("{} Generating 3 frames for GIF...", spinner),
                ExportMode::ChainPng => format!("{} Analyzing prompt & generating tiles...", spinner),
                ExportMode::Png => format!("{} Generating pixel art...", spinner),
            };
            let quote_index = (app.spinner_frame / 60) % QUOTES.len();
            let quote = QUOTES[quote_index];

            let vert = Layout::vertical([
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Percentage(40),
            ])
            .split(inner);

            let p = Paragraph::new(gen_text)
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(p, vert[1]);

            let q = Paragraph::new(format!("\"{}\"", quote))
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(q, vert[3]);
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

/// Render the canvas using half-block characters for 2x resolution.
/// Each terminal cell shows 2 vertical pixels using ▀ with fg (top) and bg (bottom).
fn render_canvas(frame: &mut Frame, area: ratatui::layout::Rect, canvas: &Canvas) {
    let canvas_h = canvas.len();
    let canvas_w = if canvas_h > 0 { canvas[0].len() } else { 0 };
    if canvas_w == 0 || canvas_h == 0 {
        return;
    }

    let available_w = area.width as usize;       // 1 char per pixel now
    let available_h = area.height as usize * 2;  // 2 pixels per row

    let scale_x = available_w as f64 / canvas_w as f64;
    let scale_y = available_h as f64 / canvas_h as f64;
    let scale = scale_x.min(scale_y).min(1.0);

    let render_w = (canvas_w as f64 * scale) as usize;
    let render_h = (canvas_h as f64 * scale) as usize;

    let offset_x = (area.width as usize - render_w) / 2;
    let offset_y = (area.height as usize - (render_h + 1) / 2) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for _ in 0..offset_y {
        lines.push(Line::from(""));
    }

    // Process 2 pixel rows per terminal row
    let mut py = 0;
    while py < render_h {
        let mut spans: Vec<Span> = Vec::new();

        if offset_x > 0 {
            spans.push(Span::raw(" ".repeat(offset_x)));
        }

        for col in 0..render_w {
            let src_x = ((col as f64 / scale) as usize).min(canvas_w - 1);

            // Top pixel
            let src_y_top = ((py as f64 / scale) as usize).min(canvas_h - 1);
            let [tr, tg, tb] = canvas[src_y_top][src_x];

            // Bottom pixel (may be out of range)
            if py + 1 < render_h {
                let src_y_bot = (((py + 1) as f64 / scale) as usize).min(canvas_h - 1);
                let [br, bg, bb] = canvas[src_y_bot][src_x];
                spans.push(Span::styled(
                    "▀",
                    Style::default()
                        .fg(Color::Rgb(tr, tg, tb))
                        .bg(Color::Rgb(br, bg, bb)),
                ));
            } else {
                spans.push(Span::styled(
                    "▀",
                    Style::default().fg(Color::Rgb(tr, tg, tb)),
                ));
            }
        }
        lines.push(Line::from(spans));
        py += 2;
    }

    let canvas_widget = Paragraph::new(lines);
    frame.render_widget(canvas_widget, area);
}
