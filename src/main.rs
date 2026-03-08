use std::io::stdout;

use crossterm::{
    cursor::Hide,
    queue,
    terminal,
};
use std::io::Write;

use ayz::app::App;
use ayz::core::dungeon::GameState;
use ayz::ui::terminal::input::TerminalInput;
use ayz::ui::terminal::renderer::TerminalRenderer;

fn main() {
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    queue!(stdout, Hide).unwrap();
    stdout.flush().unwrap();

    let state = GameState::new();
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
