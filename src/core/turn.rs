use super::types::{Direction, Position};

#[derive(Debug, Clone)]
pub enum GameCommand {
    Move(Direction),
    UseItem(usize),       // inventory index
    UseStaff(usize),      // inventory index (staff item)
    EquipWeapon(usize),   // inventory index (weapon item)
    EquipShield(usize),   // inventory index (shield item)
    Wait,
    Quit,
    OpenInventory,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerMoved { from: Position, to: Position },
    PlayerAttacked { target_name: String, damage: i32 },
    MonsterDefeated { name: String, exp: i32 },
    MonsterMoved { id: usize, from: Position, to: Position },
    MonsterAttacked { name: String, damage: i32 },
    PlayerDamaged { amount: i32, remaining_hp: i32 },
    ItemPickedUp { name: String },
    ItemUsed { name: String, effect_desc: String },
    EquipmentPickedUp { name: String },
    Equipped { name: String },
    LevelUp { new_level: i32 },
    FloorAdvance { new_floor: u32 },
    InventoryFull,
    Starving,
    GameOver,
    GameClear,
    Message(String),
    RequestInventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnError {
    InvalidInventoryIndex(usize),
}
