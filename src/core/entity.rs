use super::types::{Direction, Position};

pub struct Player {
    pub level: i32,
    pub exp: i32,
    pub symbol: char,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub pos: Position,
    pub direction: Direction,
}

impl Player {
    pub fn new() -> Self {
        Player {
            level: 1,
            exp: 0,
            symbol: '@',
            hp: 30,
            max_hp: 30,
            attack: 8,
            defense: 5,
            pos: Position::zero(),
            direction: Direction::Down,
        }
    }
}

pub struct Monster {
    pub name: String,
    pub symbol: char,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp: i32,
    pub pos: Position,
}

impl Monster {
    pub fn new(pos: Position) -> Self {
        Monster {
            name: String::from("モンスター"),
            symbol: 'M',
            hp: 10,
            attack: 3,
            defense: 2,
            exp: 10,
            pos,
        }
    }
}
