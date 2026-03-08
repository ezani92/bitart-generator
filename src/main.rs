mod generator;
mod tui;
mod exporter;

fn main() -> std::io::Result<()> {
    tui::run()
}
