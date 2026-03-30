use serde::Deserialize;
use std::collections::HashMap;

// --- Item Definitions ---

#[derive(Clone, Debug, Deserialize)]
pub enum ItemEffectDef {
    Heal(i32),
    HealFull,
    Food(i32),
    BoostAttack(i32),
    RevealMap,
    ConfuseAll { turns: u32 },
    TempBoostAttack { amount: i32, turns: u32 },
    Paralyze,
    Knockback { distance: i32 },
    SwapPosition,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ItemCategoryDef {
    Herb,
    Food,
    Scroll,
    Staff,
    Weapon,
    Shield,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub symbol: char,
    pub category: ItemCategoryDef,
    pub effect: ItemEffectDef,
}

// --- Equipment Definitions ---

#[derive(Clone, Debug, Deserialize)]
pub enum EquipCategoryDef {
    Weapon,
    Shield,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EquipmentDef {
    pub name: String,
    pub symbol: char,
    pub category: EquipCategoryDef,
    pub base_value: i32,
}

// --- Monster Definitions ---

#[derive(Clone, Debug, Deserialize)]
pub enum AiTypeDef {
    Standard,
    Ranged,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MonsterStatsDef {
    pub name: String,
    pub symbol: char,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp: i32,
    pub ai_type: AiTypeDef,
}

// --- Floor Spawn Tables ---

#[derive(Clone, Debug, Deserialize)]
pub struct FloorRange(pub u32, pub u32);

impl FloorRange {
    pub fn contains(&self, floor: u32) -> bool {
        floor >= self.0 && floor <= self.1
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MonsterTableEntry {
    pub floors: FloorRange,
    pub monsters: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MonsterCountEntry {
    pub floors: FloorRange,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HerbSpawnEntry {
    pub floors: FloorRange,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FoodSpawnEntry {
    pub floors: FloorRange,
    pub count: u32,
    pub items: Vec<FoodSpawnItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FoodSpawnItem {
    pub id: String,
    pub weight: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScrollSpawnDef {
    pub min: u32,
    pub max: u32,
    pub pool: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StaffSpawnDef {
    pub chance: f64,
    pub min_charges: i32,
    pub max_charges: i32,
    pub pool: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EquipSpawnEntry {
    pub floors: FloorRange,
    pub chance: f64,
    pub weapons: Vec<String>,
    pub shields: Vec<String>,
    pub max_enhancement: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FloorData {
    pub monster_table: Vec<MonsterTableEntry>,
    pub monster_counts: Vec<MonsterCountEntry>,
    pub herb_spawns: Vec<HerbSpawnEntry>,
    pub food_spawns: Vec<FoodSpawnEntry>,
    pub scroll_spawns: ScrollSpawnDef,
    pub staff_spawns: StaffSpawnDef,
    pub equipment_spawns: Vec<EquipSpawnEntry>,
}

// --- Player ---

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerInitialStats {
    pub level: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub fullness: i32,
    pub max_fullness: i32,
    pub symbol: char,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LevelEntry {
    pub level: i32,
    pub req_exp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerData {
    pub initial_stats: PlayerInitialStats,
    pub max_inventory: usize,
    pub level_table: Vec<LevelEntry>,
}

// --- Balance ---

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceData {
    pub min_damage: i32,
    pub thrown_non_weapon_damage: i32,
    pub fullness_decrease_per_turn: i32,
    pub starvation_damage: i32,
    pub detection_range: i32,
    pub staff_range: i32,
    pub throw_range: i32,
}

// --- Map ---

#[derive(Clone, Debug, Deserialize)]
pub struct MapData {
    pub width: usize,
    pub height: usize,
    pub min_room_size: usize,
    pub min_aisle_size: usize,
    pub cut_trial: usize,
    pub max_random_aisles: usize,
    pub max_floor: u32,
}

// --- Top-level container ---

pub struct MasterData {
    pub items: HashMap<String, ItemDef>,
    pub equipment: HashMap<String, EquipmentDef>,
    pub monsters: HashMap<String, MonsterStatsDef>,
    pub floors: FloorData,
    pub player: PlayerData,
    pub balance: BalanceData,
    pub map: MapData,
}
