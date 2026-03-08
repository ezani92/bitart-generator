mod config;
mod generator;
mod tui;
mod exporter;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut prompt = None;
    let mut output = None;
    let mut gif_mode = false;
    let mut chain_mode = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--prompt" => {
                i += 1;
                if i < args.len() {
                    prompt = Some(args[i].clone());
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(args[i].clone());
                }
            }
            "-g" | "--gif" => {
                gif_mode = true;
            }
            "-c" | "--chain" => {
                chain_mode = true;
            }
            "-h" | "--help" => {
                println!("BitArt Generator - Terminal pixel art from text prompts");
                println!();
                println!("Usage: bitart [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --prompt <TEXT>   Generate art from prompt (skip TUI)");
                println!("  -o, --output <PATH>   Output folder for chain mode, file path otherwise");
                println!("  -g, --gif             Generate animated GIF (3 frames at 3fps)");
                println!("  -c, --chain           Generate chained tile assets (multiple PNGs)");
                println!("  -h, --help            Show this help");
                println!("  -v, --version         Show version");
                println!();
                println!("Examples:");
                println!("  bitart                              Launch interactive TUI");
                println!("  bitart -p \"oak tree\"                 Generate PNG");
                println!("  bitart -p \"sunset\" -o art.png        Generate with custom path");
                println!("  bitart -p \"dancing cat\" -g           Generate animated GIF");
                println!("  bitart -p \"fire\" -g -o fire.gif      GIF with custom path");
                println!("  bitart -p \"stone wall\" -c            Generate chained tile set");
                println!("  bitart -p \"stone wall\" -c -o tiles/  Chain to custom folder");
                println!();
                println!("TUI shortcuts:");
                println!("  Shift+Tab    Toggle PNG/GIF/CHAIN mode");
                return Ok(());
            }
            "-v" | "--version" => {
                println!("bitart {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(prompt) = prompt {
        return cli_generate(&prompt, output.as_deref(), gif_mode, chain_mode);
    }

    tui::run()
}

fn cli_generate(prompt: &str, output: Option<&str>, gif_mode: bool, chain_mode: bool) -> std::io::Result<()> {
    let config = match config::Config::load() {
        Some(c) => c,
        None => {
            eprintln!("Error: No config found. Run `bitart` first to set up your API key.");
            std::process::exit(1);
        }
    };

    if chain_mode {
        let default_dir = {
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
        };
        let output_dir = output.unwrap_or(&default_dir);

        eprintln!("Analyzing & generating tiles for \"{}\"...", prompt);

        let rx = generator::generate_chain_async(
            prompt.to_string(),
            config.api_key.clone(),
            config.model.clone(),
        );

        match rx.recv() {
            Ok(Ok(result)) => {
                let tile_refs: Vec<(String, &generator::Canvas)> = result
                    .tiles
                    .iter()
                    .map(|t| (t.name.clone(), &t.canvas))
                    .collect();

                match exporter::save_chain_pngs(&tile_refs, output_dir) {
                    Ok(paths) => {
                        eprintln!("Saved {} tiles to {}:", paths.len(), output_dir);
                        for path in &paths {
                            eprintln!("  {}", path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error saving: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("Generation failed: {}", e);
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("Generation failed unexpectedly");
                std::process::exit(1);
            }
        }

        return Ok(());
    }

    if gif_mode {
        let default_path = config.output_path("gif");
        let output_path = output.unwrap_or(&default_path);

        eprintln!("Generating 3 frames for \"{}\"...", prompt);

        let rx = generator::generate_frames_async(
            prompt.to_string(),
            config.api_key.clone(),
            config.model.clone(),
        );

        match rx.recv() {
            Ok(Ok(result)) => {
                match exporter::save_gif(&result.frames, output_path) {
                    Ok(()) => eprintln!("Saved to {}", output_path),
                    Err(e) => {
                        eprintln!("Error saving: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("Generation failed: {}", e);
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("Generation failed unexpectedly");
                std::process::exit(1);
            }
        }
    } else {
        let default_path = config.output_path("png");
        let output_path = output.unwrap_or(&default_path);

        eprintln!("Generating pixel art for \"{}\"...", prompt);

        let rx = generator::generate_async(
            prompt.to_string(),
            config.api_key.clone(),
            config.model.clone(),
        );

        match rx.recv() {
            Ok(Ok(result)) => {
                match exporter::save_png(&result.canvas, output_path) {
                    Ok(()) => eprintln!("Saved to {}", output_path),
                    Err(e) => {
                        eprintln!("Error saving: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("Generation failed: {}", e);
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("Generation failed unexpectedly");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
