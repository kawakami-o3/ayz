use crate::core::turn::GameCommand;

pub trait InputHandler {
    type Error;

    fn next_command(&mut self) -> Result<GameCommand, Self::Error>;
}
