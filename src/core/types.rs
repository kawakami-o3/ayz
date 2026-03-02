#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn zero() -> Position {
        Position { x: 0, y: 0 }
    }

    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    pub fn plus(&self, other: &Position) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn manhattan_distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    pub fn to_offset(&self) -> Position {
        match self {
            Direction::Up => Position::new(0, -1),
            Direction::Down => Position::new(0, 1),
            Direction::Left => Position::new(-1, 0),
            Direction::Right => Position::new(1, 0),
            Direction::UpLeft => Position::new(-1, -1),
            Direction::UpRight => Position::new(1, -1),
            Direction::DownLeft => Position::new(-1, 1),
            Direction::DownRight => Position::new(1, 1),
        }
    }

    pub fn all() -> &'static [Direction; 8] {
        &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::UpLeft,
            Direction::UpRight,
            Direction::DownLeft,
            Direction::DownRight,
        ]
    }
}
