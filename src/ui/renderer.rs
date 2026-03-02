use crate::core::dungeon::GameState;

pub trait Renderer {
    type Error;

    fn render(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn push_message(&mut self, msg: &str);
    fn render_game_over(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn render_game_clear(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn cleanup(&mut self) -> Result<(), Self::Error>;
}
