use crate::core::dungeon::GameState;
use crate::core::entity::Item;

pub trait Renderer {
    type Error;

    fn render(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn push_message(&mut self, msg: &str);
    fn render_game_over(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn render_game_clear(&mut self, state: &GameState) -> Result<(), Self::Error>;
    fn render_inventory(&mut self, items: &[Item], weapon: Option<&str>, shield: Option<&str>) -> Result<Option<InventoryAction>, Self::Error>;
    fn cleanup(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum InventoryAction {
    Use(usize),
    Cancel,
}
