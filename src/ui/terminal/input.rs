use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::core::turn::GameCommand;
use crate::core::types::Direction;
use crate::ui::input::InputHandler;

pub struct TerminalInput;

impl TerminalInput {
    pub fn new() -> Self {
        TerminalInput
    }
}

impl InputHandler for TerminalInput {
    type Error = std::io::Error;

    fn next_command(&mut self) -> Result<GameCommand, Self::Error> {
        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                    let cmd = match key.code {
                        KeyCode::Char('q') => GameCommand::Quit,
                        // Shift+direction: turn without moving
                        KeyCode::Char('H') => GameCommand::Turn(Direction::Left),
                        KeyCode::Char('J') => GameCommand::Turn(Direction::Down),
                        KeyCode::Char('K') => GameCommand::Turn(Direction::Up),
                        KeyCode::Char('L') => GameCommand::Turn(Direction::Right),
                        KeyCode::Char('Y') => GameCommand::Turn(Direction::UpLeft),
                        KeyCode::Char('U') => GameCommand::Turn(Direction::UpRight),
                        KeyCode::Char('N') => GameCommand::Turn(Direction::DownLeft),
                        KeyCode::Char('M') => GameCommand::Turn(Direction::DownRight),
                        // Shift+arrow keys: turn without moving
                        KeyCode::Left if shift => GameCommand::Turn(Direction::Left),
                        KeyCode::Down if shift => GameCommand::Turn(Direction::Down),
                        KeyCode::Up if shift => GameCommand::Turn(Direction::Up),
                        KeyCode::Right if shift => GameCommand::Turn(Direction::Right),
                        // Normal direction: move
                        KeyCode::Char('h') | KeyCode::Left => GameCommand::Move(Direction::Left),
                        KeyCode::Char('j') | KeyCode::Down => GameCommand::Move(Direction::Down),
                        KeyCode::Char('k') | KeyCode::Up => GameCommand::Move(Direction::Up),
                        KeyCode::Char('l') | KeyCode::Right => GameCommand::Move(Direction::Right),
                        KeyCode::Char('y') => GameCommand::Move(Direction::UpLeft),
                        KeyCode::Char('u') => GameCommand::Move(Direction::UpRight),
                        KeyCode::Char('n') => GameCommand::Move(Direction::DownLeft),
                        KeyCode::Char('m') => GameCommand::Move(Direction::DownRight),
                        KeyCode::Char('.') => GameCommand::Wait,
                        KeyCode::Char('i') => GameCommand::OpenInventory,
                        KeyCode::Char('t') => GameCommand::OpenThrowInventory,
                        _ => continue,
                    };
                    return Ok(cmd);
                }
                _ => continue,
            }
        }
    }
}
