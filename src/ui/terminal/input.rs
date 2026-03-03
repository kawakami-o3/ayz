use crossterm::event::{self, Event, KeyCode, KeyEventKind};

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
                    let cmd = match key.code {
                        KeyCode::Char('q') => GameCommand::Quit,
                        KeyCode::Char('h') | KeyCode::Left => {
                            GameCommand::Move(Direction::Left)
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            GameCommand::Move(Direction::Down)
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            GameCommand::Move(Direction::Up)
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            GameCommand::Move(Direction::Right)
                        }
                        KeyCode::Char('y') => GameCommand::Move(Direction::UpLeft),
                        KeyCode::Char('u') => GameCommand::Move(Direction::UpRight),
                        KeyCode::Char('n') => GameCommand::Move(Direction::DownLeft),
                        KeyCode::Char('m') => GameCommand::Move(Direction::DownRight),
                        KeyCode::Char('.') => GameCommand::Wait,
                        KeyCode::Char('i') => GameCommand::OpenInventory,
                        _ => continue,
                    };
                    return Ok(cmd);
                }
                _ => continue,
            }
        }
    }
}
