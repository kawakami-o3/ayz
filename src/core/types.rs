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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_zero() {
        let p = Position::zero();
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }

    #[test]
    fn position_new() {
        let p = Position::new(3, -7);
        assert_eq!(p.x, 3);
        assert_eq!(p.y, -7);
    }

    #[test]
    fn position_plus() {
        let a = Position::new(2, 3);
        let b = Position::new(-1, 5);
        let c = a.plus(&b);
        assert_eq!(c, Position::new(1, 8));
    }

    #[test]
    fn position_plus_with_negative() {
        let a = Position::new(-3, -4);
        let b = Position::new(-2, 1);
        assert_eq!(a.plus(&b), Position::new(-5, -3));
    }

    #[test]
    fn manhattan_distance_same_position() {
        let p = Position::new(5, 5);
        assert_eq!(p.manhattan_distance(&p), 0);
    }

    #[test]
    fn manhattan_distance_straight() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 0);
        assert_eq!(a.manhattan_distance(&b), 3);
    }

    #[test]
    fn manhattan_distance_diagonal() {
        let a = Position::new(1, 1);
        let b = Position::new(4, 5);
        assert_eq!(a.manhattan_distance(&b), 7); // |3| + |4|
    }

    #[test]
    fn direction_to_offset_cardinal() {
        assert_eq!(Direction::Up.to_offset(), Position::new(0, -1));
        assert_eq!(Direction::Down.to_offset(), Position::new(0, 1));
        assert_eq!(Direction::Left.to_offset(), Position::new(-1, 0));
        assert_eq!(Direction::Right.to_offset(), Position::new(1, 0));
    }

    #[test]
    fn direction_to_offset_diagonal() {
        assert_eq!(Direction::UpLeft.to_offset(), Position::new(-1, -1));
        assert_eq!(Direction::UpRight.to_offset(), Position::new(1, -1));
        assert_eq!(Direction::DownLeft.to_offset(), Position::new(-1, 1));
        assert_eq!(Direction::DownRight.to_offset(), Position::new(1, 1));
    }

    #[test]
    fn direction_all_returns_eight() {
        assert_eq!(Direction::all().len(), 8);
    }
}
