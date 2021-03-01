#[derive(Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn plus(&self, pos: &Position) -> Position {
        Position {
            x: self.x + pos.x,
            y: self.y + pos.y,
        }
    }
}

