use super::types::{Direction, Position};

// --- Items ---

#[derive(Clone, Debug)]
pub enum ItemEffect {
    Heal(i32),
    HealFull,
    Food(i32),        // fullness recovery (internal: 0-1000)
    BoostAttack(i32), // permanent attack boost
    RevealMap,
    ConfuseAll { turns: u32 },
    TempBoostAttack { amount: i32, turns: u32 },
    Paralyze,
    Knockback { distance: i32 },
    SwapPosition,
}

#[derive(Clone, Debug)]
pub enum ItemCategory {
    Herb,
    Food,
    Weapon,
    Shield,
    Scroll,
    Staff(i32), // charges remaining
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

    // Scrolls
    pub fn light_scroll() -> Self {
        Item {
            id: "light_scroll".into(),
            name: "あかりの巻物".into(),
            symbol: '?',
            category: ItemCategory::Scroll,
            effect: ItemEffect::RevealMap,
            equip_data: None,
        }
    }

    pub fn confusion_scroll() -> Self {
        Item {
            id: "confusion_scroll".into(),
            name: "混乱の巻物".into(),
            symbol: '?',
            category: ItemCategory::Scroll,
            effect: ItemEffect::ConfuseAll { turns: 10 },
            equip_data: None,
        }
    }

    pub fn power_scroll() -> Self {
        Item {
            id: "power_scroll".into(),
            name: "パワーアップの巻物".into(),
            symbol: '?',
            category: ItemCategory::Scroll,
            effect: ItemEffect::TempBoostAttack { amount: 5, turns: 20 },
            equip_data: None,
        }
    }

    // Staffs
    pub fn paralysis_staff(charges: i32) -> Self {
        Item {
            id: "paralysis_staff".into(),
            name: "かなしばりの杖".into(),
            symbol: '/',
            category: ItemCategory::Staff(charges),
            effect: ItemEffect::Paralyze,
            equip_data: None,
        }
    }

    pub fn knockback_staff(charges: i32) -> Self {
        Item {
            id: "knockback_staff".into(),
            name: "ふきとばしの杖".into(),
            symbol: '/',
            category: ItemCategory::Staff(charges),
            effect: ItemEffect::Knockback { distance: 5 },
            equip_data: None,
        }
    }

    pub fn swap_staff(charges: i32) -> Self {
        Item {
            id: "swap_staff".into(),
            name: "場所がえの杖".into(),
            symbol: '/',
            category: ItemCategory::Staff(charges),
            effect: ItemEffect::SwapPosition,
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

// --- Status Effects ---

#[derive(Clone, Debug, Default)]
pub struct StatusEffects {
    pub confused: Option<u32>,   // remaining turns
    pub paralyzed: bool,
    pub attack_boost: Option<(i32, u32)>, // (amount, remaining turns)
}

impl StatusEffects {
    pub fn tick(&mut self) {
        if let Some(t) = &mut self.confused {
            if *t <= 1 {
                self.confused = None;
            } else {
                *t -= 1;
            }
        }
        if let Some((_, t)) = &mut self.attack_boost {
            if *t <= 1 {
                self.attack_boost = None;
            } else {
                *t -= 1;
            }
        }
    }
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
    pub status: StatusEffects,
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
            status: StatusEffects::default(),
        }
    }

    pub fn effective_attack(&self) -> i32 {
        let base = self.attack + self.weapon.as_ref().map_or(0, |w| w.effective_value());
        let boost = self.status.attack_boost.map_or(0, |(amt, _)| amt);
        base + boost
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
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp: i32,
    pub pos: Position,
    pub ai_type: AiType,
    pub status: StatusEffects,
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
            max_hp: def.hp,
            attack: def.attack,
            defense: def.defense,
            exp: def.exp,
            pos,
            ai_type: def.ai_type.clone(),
            status: StatusEffects::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Equipment ---

    #[test]
    fn equipment_display_name_no_enhancement() {
        let e = Equipment::wooden_sword(0);
        assert_eq!(e.display_name(), "木の剣");
    }

    #[test]
    fn equipment_display_name_with_enhancement() {
        let e = Equipment::iron_sword(3);
        assert_eq!(e.display_name(), "鉄の剣+3");
    }

    #[test]
    fn equipment_effective_value() {
        let e = Equipment::wooden_sword(2);
        assert_eq!(e.effective_value(), 3 + 2); // base 3 + enhancement 2
    }

    #[test]
    fn weapon_factories() {
        let w = Equipment::wooden_sword(0);
        assert_eq!(w.base_value, 3);
        assert_eq!(w.category, EquipCategory::Weapon);

        let w = Equipment::iron_sword(0);
        assert_eq!(w.base_value, 6);

        let w = Equipment::steel_sword(0);
        assert_eq!(w.base_value, 10);
    }

    #[test]
    fn shield_factories() {
        let s = Equipment::wooden_shield(0);
        assert_eq!(s.base_value, 3);
        assert_eq!(s.category, EquipCategory::Shield);

        let s = Equipment::iron_shield(0);
        assert_eq!(s.base_value, 5);

        let s = Equipment::steel_shield(0);
        assert_eq!(s.base_value, 9);
    }

    // --- StatusEffects ---

    #[test]
    fn status_effects_tick_confused_decrements() {
        let mut status = StatusEffects {
            confused: Some(3),
            paralyzed: false,
            attack_boost: None,
        };
        status.tick();
        assert_eq!(status.confused, Some(2));
        status.tick();
        assert_eq!(status.confused, Some(1));
        status.tick();
        assert_eq!(status.confused, None); // <= 1 clears it
    }

    #[test]
    fn status_effects_tick_attack_boost_decrements() {
        let mut status = StatusEffects {
            confused: None,
            paralyzed: false,
            attack_boost: Some((5, 2)),
        };
        status.tick();
        assert_eq!(status.attack_boost, Some((5, 1)));
        status.tick();
        assert_eq!(status.attack_boost, None);
    }

    #[test]
    fn status_effects_tick_paralyzed_unchanged() {
        let mut status = StatusEffects {
            confused: None,
            paralyzed: true,
            attack_boost: None,
        };
        status.tick();
        assert!(status.paralyzed); // tick does not clear paralyzed
    }

    // --- Player ---

    #[test]
    fn player_new_initial_stats() {
        let p = Player::new();
        assert_eq!(p.level, 1);
        assert_eq!(p.hp, 30);
        assert_eq!(p.max_hp, 30);
        assert_eq!(p.attack, 8);
        assert_eq!(p.defense, 5);
        assert_eq!(p.fullness, 1000);
        assert!(p.weapon.is_none());
        assert!(p.shield.is_none());
        assert!(p.inventory.is_empty());
    }

    #[test]
    fn player_effective_attack_bare_hands() {
        let p = Player::new();
        assert_eq!(p.effective_attack(), 8); // base attack only
    }

    #[test]
    fn player_effective_attack_with_weapon() {
        let mut p = Player::new();
        p.weapon = Some(Equipment::iron_sword(2)); // base 6 + enhancement 2 = 8
        assert_eq!(p.effective_attack(), 8 + 8); // player attack + weapon
    }

    #[test]
    fn player_effective_attack_with_boost() {
        let mut p = Player::new();
        p.weapon = Some(Equipment::wooden_sword(0)); // base 3
        p.status.attack_boost = Some((5, 10));
        assert_eq!(p.effective_attack(), 8 + 3 + 5); // player + weapon + boost
    }

    #[test]
    fn player_effective_defense_no_shield() {
        let p = Player::new();
        assert_eq!(p.effective_defense(), 5);
    }

    #[test]
    fn player_effective_defense_with_shield() {
        let mut p = Player::new();
        p.shield = Some(Equipment::iron_shield(1)); // base 5 + 1 = 6
        assert_eq!(p.effective_defense(), 5 + 6);
    }

    #[test]
    fn player_inventory_full() {
        let mut p = Player::new();
        assert!(!p.inventory_full());
        for _ in 0..MAX_INVENTORY {
            p.inventory.push(Item::herb());
        }
        assert!(p.inventory_full());
    }

    // --- monsters_for_floor ---

    #[test]
    fn monsters_for_floor_early() {
        let m = monsters_for_floor(1);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].id, "slime");
        assert_eq!(m[1].id, "goblin");
    }

    #[test]
    fn monsters_for_floor_mid() {
        let m = monsters_for_floor(3);
        assert_eq!(m.len(), 3); // slime, goblin, bat

        let m = monsters_for_floor(5);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "goblin");
    }

    #[test]
    fn monsters_for_floor_late() {
        let m = monsters_for_floor(10);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "drake");
        assert_eq!(m[2].id, "guardian");
    }

    // --- Monster ---

    #[test]
    fn monster_from_def() {
        let def = &monsters_for_floor(1)[0]; // slime
        let pos = Position::new(5, 5);
        let monster = Monster::from_def(def, pos);
        assert_eq!(monster.name, "スライム");
        assert_eq!(monster.hp, 5);
        assert_eq!(monster.pos, pos);
        assert!(monster.status.confused.is_none());
        assert!(!monster.status.paralyzed);
    }

    // === 境界条件テスト ===

    // #3: display_name の enhancement 負値
    #[test]
    fn equipment_display_name_negative_enhancement() {
        let e = Equipment {
            id: "cursed_sword".into(),
            name: "呪いの剣".into(),
            symbol: ')',
            category: EquipCategory::Weapon,
            base_value: 5,
            enhancement: -1,
        };
        // 現在の実装: enhancement <= 0 なら名前のみ返す
        assert_eq!(e.display_name(), "呪いの剣");
    }

    // #4: inventory_full の境界値19
    #[test]
    fn player_inventory_not_full_at_19() {
        let mut p = Player::new();
        for _ in 0..19 {
            p.inventory.push(Item::herb());
        }
        assert!(!p.inventory_full()); // 19 < 20
    }

    // #10: monsters_for_floor の全境界値
    #[test]
    fn monsters_for_floor_boundary_floor_0() {
        // floor=0 は match の _ アームに該当
        let m = monsters_for_floor(0);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "drake");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_2() {
        let m = monsters_for_floor(2);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].id, "slime");
        assert_eq!(m[1].id, "goblin");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_4() {
        let m = monsters_for_floor(4);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "goblin");
        assert_eq!(m[1].id, "bat");
        assert_eq!(m[2].id, "golem");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_6() {
        let m = monsters_for_floor(6);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "specter");
        assert_eq!(m[1].id, "golem");
        assert_eq!(m[2].id, "imp");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_7() {
        let m = monsters_for_floor(7);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "specter");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_8() {
        let m = monsters_for_floor(8);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "specter");
        assert_eq!(m[1].id, "drake");
        assert_eq!(m[2].id, "imp");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_9() {
        let m = monsters_for_floor(9);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "specter");
    }

    #[test]
    fn monsters_for_floor_boundary_floor_100() {
        // Very high floor: should use _ arm
        let m = monsters_for_floor(100);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].id, "drake");
        assert_eq!(m[1].id, "imp");
        assert_eq!(m[2].id, "guardian");
    }
}
