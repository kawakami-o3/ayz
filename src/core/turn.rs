use super::types::{Direction, Position};

#[derive(Debug, Clone)]
pub enum GameCommand {
    Move(Direction),
    Wait,
    Quit,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerMoved { from: Position, to: Position },
    PlayerAttacked { target_name: String, damage: i32 },
    MonsterDefeated { name: String, exp: i32 },
    MonsterMoved { id: usize, from: Position, to: Position },
    MonsterAttacked { name: String, damage: i32 },
    PlayerDamaged { amount: i32, remaining_hp: i32 },
    LevelUp { new_level: i32 },
    FloorAdvance { new_floor: u32 },
    GameOver,
    GameClear,
    Message(String),
}
