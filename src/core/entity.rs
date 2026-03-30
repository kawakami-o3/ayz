use super::master_data::{
    AiTypeDef, EquipCategoryDef, EquipmentDef, ItemCategoryDef, ItemDef, ItemEffectDef,
    MonsterStatsDef, PlayerData,
};
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

impl ItemEffect {
    pub fn from_def(def: &ItemEffectDef) -> Self {
        match def {
            ItemEffectDef::Heal(v) => ItemEffect::Heal(*v),
            ItemEffectDef::HealFull => ItemEffect::HealFull,
            ItemEffectDef::Food(v) => ItemEffect::Food(*v),
            ItemEffectDef::BoostAttack(v) => ItemEffect::BoostAttack(*v),
            ItemEffectDef::RevealMap => ItemEffect::RevealMap,
            ItemEffectDef::ConfuseAll { turns } => ItemEffect::ConfuseAll { turns: *turns },
            ItemEffectDef::TempBoostAttack { amount, turns } => ItemEffect::TempBoostAttack {
                amount: *amount,
                turns: *turns,
            },
            ItemEffectDef::Paralyze => ItemEffect::Paralyze,
            ItemEffectDef::Knockback { distance } => ItemEffect::Knockback {
                distance: *distance,
            },
            ItemEffectDef::SwapPosition => ItemEffect::SwapPosition,
        }
    }
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
    pub fn from_def(id: &str, def: &ItemDef, charges: Option<i32>) -> Self {
        let category = match (&def.category, charges) {
            (ItemCategoryDef::Staff, Some(c)) => ItemCategory::Staff(c),
            (ItemCategoryDef::Staff, None) => ItemCategory::Staff(0),
            (ItemCategoryDef::Herb, _) => ItemCategory::Herb,
            (ItemCategoryDef::Food, _) => ItemCategory::Food,
            (ItemCategoryDef::Scroll, _) => ItemCategory::Scroll,
            (ItemCategoryDef::Weapon, _) => ItemCategory::Weapon,
            (ItemCategoryDef::Shield, _) => ItemCategory::Shield,
        };
        Item {
            id: id.to_string(),
            name: def.name.clone(),
            symbol: def.symbol,
            category,
            effect: ItemEffect::from_def(&def.effect),
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

    pub fn from_def(id: &str, def: &EquipmentDef, enhancement: i32) -> Self {
        Equipment {
            id: id.to_string(),
            name: def.name.clone(),
            symbol: def.symbol,
            category: match def.category {
                EquipCategoryDef::Weapon => EquipCategory::Weapon,
                EquipCategoryDef::Shield => EquipCategory::Shield,
            },
            base_value: def.base_value,
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
    pub confused: Option<u32>, // remaining turns
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
    pub fn new(data: &PlayerData) -> Self {
        let s = &data.initial_stats;
        Player {
            level: s.level,
            exp: 0,
            symbol: s.symbol,
            hp: s.hp,
            max_hp: s.hp,
            attack: s.attack,
            defense: s.defense,
            pos: Position::zero(),
            direction: Direction::Down,
            inventory: Vec::new(),
            weapon: None,
            shield: None,
            fullness: s.fullness,
            max_fullness: s.max_fullness,
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

    pub fn inventory_full(&self, max_inventory: usize) -> bool {
        self.inventory.len() >= max_inventory
    }
}

// --- Monster ---

#[derive(Clone, Debug)]
pub enum AiType {
    Standard,
    Ranged,
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

impl Monster {
    pub fn from_stats_def(def: &MonsterStatsDef, pos: Position) -> Self {
        Monster {
            name: def.name.clone(),
            symbol: def.symbol,
            hp: def.hp,
            max_hp: def.hp,
            attack: def.attack,
            defense: def.defense,
            exp: def.exp,
            pos,
            ai_type: match &def.ai_type {
                AiTypeDef::Standard => AiType::Standard,
                AiTypeDef::Ranged => AiType::Ranged,
            },
            status: StatusEffects::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::master_data::*;

    fn test_player_data() -> PlayerData {
        PlayerData {
            initial_stats: PlayerInitialStats {
                level: 1,
                hp: 30,
                attack: 8,
                defense: 5,
                fullness: 1000,
                max_fullness: 1000,
                symbol: '@',
            },
            max_inventory: 20,
            level_table: vec![],
        }
    }

    fn test_item_def(name: &str, category: ItemCategoryDef, effect: ItemEffectDef) -> ItemDef {
        ItemDef {
            name: name.to_string(),
            symbol: '!',
            category,
            effect,
        }
    }

    fn test_equip_def(name: &str, category: EquipCategoryDef, base_value: i32) -> EquipmentDef {
        EquipmentDef {
            name: name.to_string(),
            symbol: ')',
            category,
            base_value,
        }
    }

    fn test_monster_def(
        name: &str,
        hp: i32,
        attack: i32,
        defense: i32,
        exp: i32,
    ) -> MonsterStatsDef {
        MonsterStatsDef {
            name: name.to_string(),
            symbol: 's',
            hp,
            attack,
            defense,
            exp,
            ai_type: AiTypeDef::Standard,
        }
    }

    // --- Equipment ---

    #[test]
    fn equipment_display_name_no_enhancement() {
        let def = test_equip_def("木の剣", EquipCategoryDef::Weapon, 3);
        let e = Equipment::from_def("wooden_sword", &def, 0);
        assert_eq!(e.display_name(), "木の剣");
    }

    #[test]
    fn equipment_display_name_with_enhancement() {
        let def = test_equip_def("鉄の剣", EquipCategoryDef::Weapon, 6);
        let e = Equipment::from_def("iron_sword", &def, 3);
        assert_eq!(e.display_name(), "鉄の剣+3");
    }

    #[test]
    fn equipment_effective_value() {
        let def = test_equip_def("木の剣", EquipCategoryDef::Weapon, 3);
        let e = Equipment::from_def("wooden_sword", &def, 2);
        assert_eq!(e.effective_value(), 3 + 2);
    }

    #[test]
    fn weapon_from_def() {
        let def = test_equip_def("木の剣", EquipCategoryDef::Weapon, 3);
        let w = Equipment::from_def("wooden_sword", &def, 0);
        assert_eq!(w.base_value, 3);
        assert_eq!(w.category, EquipCategory::Weapon);

        let def = test_equip_def("鉄の剣", EquipCategoryDef::Weapon, 6);
        let w = Equipment::from_def("iron_sword", &def, 0);
        assert_eq!(w.base_value, 6);

        let def = test_equip_def("鋼の剣", EquipCategoryDef::Weapon, 10);
        let w = Equipment::from_def("steel_sword", &def, 0);
        assert_eq!(w.base_value, 10);
    }

    #[test]
    fn shield_from_def() {
        let def = test_equip_def("木の盾", EquipCategoryDef::Shield, 3);
        let s = Equipment::from_def("wooden_shield", &def, 0);
        assert_eq!(s.base_value, 3);
        assert_eq!(s.category, EquipCategory::Shield);

        let def = test_equip_def("鉄の盾", EquipCategoryDef::Shield, 5);
        let s = Equipment::from_def("iron_shield", &def, 0);
        assert_eq!(s.base_value, 5);

        let def = test_equip_def("鋼の盾", EquipCategoryDef::Shield, 9);
        let s = Equipment::from_def("steel_shield", &def, 0);
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
        assert_eq!(status.confused, None);
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
        assert!(status.paralyzed);
    }

    // --- Player ---

    #[test]
    fn player_new_initial_stats() {
        let data = test_player_data();
        let p = Player::new(&data);
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
        let data = test_player_data();
        let p = Player::new(&data);
        assert_eq!(p.effective_attack(), 8);
    }

    #[test]
    fn player_effective_attack_with_weapon() {
        let data = test_player_data();
        let mut p = Player::new(&data);
        let def = test_equip_def("鉄の剣", EquipCategoryDef::Weapon, 6);
        p.weapon = Some(Equipment::from_def("iron_sword", &def, 2));
        assert_eq!(p.effective_attack(), 8 + 8);
    }

    #[test]
    fn player_effective_attack_with_boost() {
        let data = test_player_data();
        let mut p = Player::new(&data);
        let def = test_equip_def("木の剣", EquipCategoryDef::Weapon, 3);
        p.weapon = Some(Equipment::from_def("wooden_sword", &def, 0));
        p.status.attack_boost = Some((5, 10));
        assert_eq!(p.effective_attack(), 8 + 3 + 5);
    }

    #[test]
    fn player_effective_defense_no_shield() {
        let data = test_player_data();
        let p = Player::new(&data);
        assert_eq!(p.effective_defense(), 5);
    }

    #[test]
    fn player_effective_defense_with_shield() {
        let data = test_player_data();
        let mut p = Player::new(&data);
        let def = test_equip_def("鉄の盾", EquipCategoryDef::Shield, 5);
        p.shield = Some(Equipment::from_def("iron_shield", &def, 1));
        assert_eq!(p.effective_defense(), 5 + 6);
    }

    #[test]
    fn player_inventory_full() {
        let data = test_player_data();
        let mut p = Player::new(&data);
        assert!(!p.inventory_full(data.max_inventory));
        let herb_def = test_item_def("回復草", ItemCategoryDef::Herb, ItemEffectDef::Heal(25));
        for _ in 0..data.max_inventory {
            p.inventory.push(Item::from_def("herb", &herb_def, None));
        }
        assert!(p.inventory_full(data.max_inventory));
    }

    // --- Monster ---

    #[test]
    fn monster_from_stats_def() {
        let def = test_monster_def("スライム", 5, 2, 3, 4);
        let pos = Position::new(5, 5);
        let monster = Monster::from_stats_def(&def, pos);
        assert_eq!(monster.name, "スライム");
        assert_eq!(monster.hp, 5);
        assert_eq!(monster.pos, pos);
        assert!(monster.status.confused.is_none());
        assert!(!monster.status.paralyzed);
    }

    // === 境界条件テスト ===

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
        assert_eq!(e.display_name(), "呪いの剣");
    }

    #[test]
    fn player_inventory_not_full_at_19() {
        let data = test_player_data();
        let mut p = Player::new(&data);
        let herb_def = test_item_def("回復草", ItemCategoryDef::Herb, ItemEffectDef::Heal(25));
        for _ in 0..19 {
            p.inventory.push(Item::from_def("herb", &herb_def, None));
        }
        assert!(!p.inventory_full(data.max_inventory));
    }
}
