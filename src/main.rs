mod config;
mod generator;
mod tui;
mod exporter;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut prompt = None;
    let mut output = None;
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
            "-h" | "--help" => {
                println!("BitArt Generator - Terminal pixel art from text prompts");
                println!();
                println!("Usage: bitart [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --prompt <TEXT>   Generate art from prompt (skip TUI)");
                println!("  -o, --output <PATH>   Output PNG path (default: output.png)");
                println!("  -h, --help            Show this help");
                println!("  -v, --version         Show version");
                println!();
                println!("Examples:");
                println!("  bitart                        Launch interactive TUI");
                println!("  bitart -p \"oak tree\"           Generate and save to output.png");
                println!("  bitart -p \"sunset\" -o art.png  Generate and save to art.png");
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
        return cli_generate(&prompt, output.as_deref());
    }

    tui::run()
}

fn cli_generate(prompt: &str, output: Option<&str>) -> std::io::Result<()> {
    let config = match config::Config::load() {
        Some(c) => c,
        None => {
            eprintln!("Error: No config found. Run `bitart` first to set up your API key.");
            std::process::exit(1);
        }
    };

    let output_path = match output {
        Some(p) => p.to_string(),
        None => "output.png".to_string(),
    };

    eprintln!("Generating pixel art for \"{}\"...", prompt);

    let rx = generator::generate_async(
        prompt.to_string(),
        config.api_key.clone(),
        config.model.clone(),
    );

    match rx.recv() {
        Ok(Ok(result)) => {
            match exporter::save_png(&result.canvas, &output_path) {
                Ok(()) => {
                    eprintln!("Saved to {}", output_path);
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

    Ok(())
}
