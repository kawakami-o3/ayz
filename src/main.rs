use std::io::stdout;
use std::sync::Arc;

use crossterm::{cursor::Hide, queue, terminal};
use std::io::Write;

use ayz::app::App;
use ayz::core::data_loader::{load_master_data, resolve_data_dir};
use ayz::core::dungeon::GameState;
use ayz::ui::terminal::input::TerminalInput;
use ayz::ui::terminal::renderer::TerminalRenderer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data_dir_arg = args
        .iter()
        .position(|a| a == "--data-dir")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()));

    let data_dir = resolve_data_dir(data_dir_arg);
    let data = match load_master_data(&data_dir) {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            eprintln!("Data directory: {}", data_dir.display());
            std::process::exit(1);
        }
    };

    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    queue!(stdout, Hide).unwrap();
    stdout.flush().unwrap();

    let state = GameState::new(data);
    let renderer = TerminalRenderer::new(stdout);
    let input = TerminalInput::new();

    let mut app = App::new(state, renderer, input);

    if let Err(e) = app.run() {
        terminal::disable_raw_mode().unwrap();
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    terminal::disable_raw_mode().unwrap();
}
