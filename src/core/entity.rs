use super::types::{Direction, Position};

// --- Items ---

#[derive(Clone, Debug)]
pub enum ItemEffect {
    Heal(i32),
    HealFull,
    Food(i32),        // fullness recovery (internal: 0-1000)
    BoostAttack(i32), // permanent attack boost
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemCategory {
    Herb,
    Food,
    Weapon,
    Shield,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub symbol: char,
    pub category: ItemCategory,
    pub effect: ItemEffect,
    pub equip_data: Option<EquipData>,
}

#[derive(Clone, Debug)]
pub struct EquipData {
    pub base_value: i32,
    pub enhancement: i32,
}

impl Item {
    pub fn herb() -> Self {
        Item {
            id: "herb".into(),
            name: "回復草".into(),
            symbol: '!',
            category: ItemCategory::Herb,
            effect: ItemEffect::Heal(25),
            equip_data: None,
        }
    }

    pub fn full_heal_herb() -> Self {
        Item {
            id: "full_heal_herb".into(),
            name: "万能草".into(),
            symbol: '!',
            category: ItemCategory::Herb,
            effect: ItemEffect::HealFull,
            equip_data: None,
        }
    }

    pub fn power_herb() -> Self {
        Item {
            id: "power_herb".into(),
            name: "力の草".into(),
            symbol: '!',
            category: ItemCategory::Herb,
            effect: ItemEffect::BoostAttack(1),
            equip_data: None,
        }
    }

    pub fn ration() -> Self {
        Item {
            id: "ration".into(),
            name: "携帯食".into(),
            symbol: '%',
            category: ItemCategory::Food,
            effect: ItemEffect::Food(500),
            equip_data: None,
        }
    }

    pub fn big_ration() -> Self {
        Item {
            id: "big_ration".into(),
            name: "保存食".into(),
            symbol: '%',
            category: ItemCategory::Food,
            effect: ItemEffect::Food(1000),
            equip_data: None,
        }
    }
}

pub struct FloorItem {
    pub item: Item,
    pub pos: Position,
}

// --- Equipment ---

#[derive(Clone, Debug)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub symbol: char,
    pub category: EquipCategory,
    pub base_value: i32,
    pub enhancement: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipCategory {
    Weapon,
    Shield,
}

impl Equipment {
    pub fn display_name(&self) -> String {
        if self.enhancement > 0 {
            format!("{}+{}", self.name, self.enhancement)
        } else {
            self.name.clone()
        }
    }

    pub fn effective_value(&self) -> i32 {
        self.base_value + self.enhancement
    }

    // Weapon definitions
    pub fn wooden_sword(enhancement: i32) -> Self {
        Equipment {
            id: "wooden_sword".into(),
            name: "木の剣".into(),
            symbol: ')',
            category: EquipCategory::Weapon,
            base_value: 3,
            enhancement,
        }
    }

    pub fn iron_sword(enhancement: i32) -> Self {
        Equipment {
            id: "iron_sword".into(),
            name: "鉄の剣".into(),
            symbol: ')',
            category: EquipCategory::Weapon,
            base_value: 6,
            enhancement,
        }
    }

    pub fn steel_sword(enhancement: i32) -> Self {
        Equipment {
            id: "steel_sword".into(),
            name: "鋼の剣".into(),
            symbol: ')',
            category: EquipCategory::Weapon,
            base_value: 10,
            enhancement,
        }
    }

    // Shield definitions
    pub fn wooden_shield(enhancement: i32) -> Self {
        Equipment {
            id: "wooden_shield".into(),
            name: "木の盾".into(),
            symbol: '[',
            category: EquipCategory::Shield,
            base_value: 3,
            enhancement,
        }
    }

    pub fn iron_shield(enhancement: i32) -> Self {
        Equipment {
            id: "iron_shield".into(),
            name: "鉄の盾".into(),
            symbol: '[',
            category: EquipCategory::Shield,
            base_value: 5,
            enhancement,
        }
    }

    pub fn steel_shield(enhancement: i32) -> Self {
        Equipment {
            id: "steel_shield".into(),
            name: "鋼の盾".into(),
            symbol: '[',
            category: EquipCategory::Shield,
            base_value: 9,
            enhancement,
        }
    }
}

pub struct FloorEquipment {
    pub equip: Equipment,
    pub pos: Position,
}

// --- Player ---

pub const MAX_INVENTORY: usize = 20;

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
    pub inventory: Vec<Item>,
    pub weapon: Option<Equipment>,
    pub shield: Option<Equipment>,
    pub fullness: i32,
    pub max_fullness: i32,
    pub kill_count: u32,
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
            inventory: Vec::new(),
            weapon: None,
            shield: None,
            fullness: 1000,
            max_fullness: 1000,
            kill_count: 0,
        }
    }

    pub fn effective_attack(&self) -> i32 {
        self.attack + self.weapon.as_ref().map_or(0, |w| w.effective_value())
    }

    pub fn effective_defense(&self) -> i32 {
        self.defense + self.shield.as_ref().map_or(0, |s| s.effective_value())
    }

    pub fn inventory_full(&self) -> bool {
        self.inventory.len() >= MAX_INVENTORY
    }
}

// --- Monster ---

#[derive(Clone, Debug)]
pub enum AiType {
    Standard,
    Ranged,
}

#[derive(Clone, Debug)]
pub struct MonsterDef {
    pub id: &'static str,
    pub name: &'static str,
    pub symbol: char,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp: i32,
    pub ai_type: AiType,
}

pub struct Monster {
    pub name: String,
    pub symbol: char,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp: i32,
    pub pos: Position,
    pub ai_type: AiType,
}

// Floor table: which monsters appear on which floors
pub fn monsters_for_floor(floor: u32) -> Vec<MonsterDef> {
    const SLIME: MonsterDef = MonsterDef {
        id: "slime", name: "スライム", symbol: 's',
        hp: 5, attack: 2, defense: 3, exp: 4, ai_type: AiType::Standard,
    };
    const GOBLIN: MonsterDef = MonsterDef {
        id: "goblin", name: "ゴブリン", symbol: 'g',
        hp: 12, attack: 5, defense: 4, exp: 8, ai_type: AiType::Standard,
    };
    const BAT: MonsterDef = MonsterDef {
        id: "bat", name: "コウモリ", symbol: 'b',
        hp: 8, attack: 4, defense: 5, exp: 10, ai_type: AiType::Standard,
    };
    const SPECTER: MonsterDef = MonsterDef {
        id: "specter", name: "スペクター", symbol: 'S',
        hp: 22, attack: 11, defense: 8, exp: 18, ai_type: AiType::Standard,
    };
    const GOLEM: MonsterDef = MonsterDef {
        id: "golem", name: "ゴーレム", symbol: 'G',
        hp: 20, attack: 8, defense: 6, exp: 15, ai_type: AiType::Standard,
    };
    const DRAKE: MonsterDef = MonsterDef {
        id: "drake", name: "ドレイク", symbol: 'D',
        hp: 45, attack: 20, defense: 15, exp: 50, ai_type: AiType::Standard,
    };
    const IMP: MonsterDef = MonsterDef {
        id: "imp", name: "インプ", symbol: 'i',
        hp: 18, attack: 10, defense: 7, exp: 20, ai_type: AiType::Ranged,
    };
    const GUARDIAN: MonsterDef = MonsterDef {
        id: "guardian", name: "ガーディアン", symbol: 'W',
        hp: 60, attack: 25, defense: 30, exp: 100, ai_type: AiType::Standard,
    };

    match floor {
        1..=2 => vec![SLIME, GOBLIN],
        3 => vec![SLIME, GOBLIN, BAT],
        4..=5 => vec![GOBLIN, BAT, GOLEM],
        6..=7 => vec![SPECTER, GOLEM, IMP],
        8..=9 => vec![SPECTER, DRAKE, IMP],
        _ => vec![DRAKE, IMP, GUARDIAN],
    }
}

impl Monster {
    pub fn from_def(def: &MonsterDef, pos: Position) -> Self {
        Monster {
            name: def.name.to_string(),
            symbol: def.symbol,
            hp: def.hp,
            attack: def.attack,
            defense: def.defense,
            exp: def.exp,
            pos,
            ai_type: def.ai_type.clone(),
        }
    }
}
