use std::collections::HashSet;

use rand::prelude::*;

use super::entity::*;
use super::entity::monsters_for_floor;
use super::turn::{GameCommand, GameEvent, TurnError};
use super::types::{Direction, Position};
use super::visibility::Visibility;
use crate::map::cell::GameMap;
use crate::map::generator;
use crate::map::spawn::calc_spawn_pos;

pub struct GameState {
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub floor_items: Vec<FloorItem>,
    pub floor_equips: Vec<FloorEquipment>,
    pub map: GameMap,
    pub visibility: Visibility,
    pub floor: u32,
    pub max_floor: u32,
    pub turn: u32,
}

impl GameState {
    pub fn new() -> Self {
        let map = generator::generate();
        let mut state = GameState {
            player: Player::new(),
            monsters: Vec::new(),
            floor_items: Vec::new(),
            floor_equips: Vec::new(),
            map,
            visibility: Visibility::new(),
            floor: 1,
            max_floor: 10,
            turn: 0,
        };
        state.spawn_entities();
        state.visibility.update(&state.player.pos, &state.map);
        state
    }

    fn monster_count_for_floor(floor: u32) -> u32 {
        match floor {
            1..=3 => 8,
            4..=6 => 10,
            7..=9 => 12,
            _ => 15,
        }
    }

    fn spawn_entities(&mut self) {
        let monster_count = Self::monster_count_for_floor(self.floor);
        let mut occupied = HashSet::new();

        // Spawn monsters from floor table
        self.monsters.clear();
        let monster_pool = monsters_for_floor(self.floor);
        let mut rng = thread_rng();
        for _ in 0..monster_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let def = &monster_pool[rng.gen_range(0..monster_pool.len())];
            self.monsters.push(Monster::from_def(def, pos));
        }

        // Spawn items (herbs)
        self.floor_items.clear();
        let herb_count = 3;
        for _ in 0..herb_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            self.floor_items.push(FloorItem {
                item: Item::herb(),
                pos,
            });
        }

        // Spawn food
        let food_count = if self.floor <= 3 { 2 } else { 1 };
        for _ in 0..food_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let item = if thread_rng().gen_bool(0.3) {
                Item::big_ration()
            } else {
                Item::ration()
            };
            self.floor_items.push(FloorItem { item, pos });
        }

        // Spawn scrolls (1-2 per floor)
        let scroll_count = rng.gen_range(1..=2);
        for _ in 0..scroll_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let scroll = match rng.gen_range(0..3) {
                0 => Item::light_scroll(),
                1 => Item::confusion_scroll(),
                _ => Item::power_scroll(),
            };
            self.floor_items.push(FloorItem { item: scroll, pos });
        }

        // Spawn staffs (0-1 per floor)
        if rng.gen_bool(0.5) {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let charges = rng.gen_range(3..=6);
            let staff = match rng.gen_range(0..3) {
                0 => Item::paralysis_staff(charges),
                1 => Item::knockback_staff(charges),
                _ => Item::swap_staff(charges),
            };
            self.floor_items.push(FloorItem { item: staff, pos });
        }

        // Spawn equipment (1 weapon or shield per floor, with chance)
        self.floor_equips.clear();
        let mut rng = thread_rng();
        if rng.gen_bool(0.5) {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let enhancement = rng.gen_range(0..=3);
            let equip = match self.floor {
                1..=3 => {
                    if rng.gen_bool(0.5) {
                        Equipment::wooden_sword(enhancement)
                    } else {
                        Equipment::wooden_shield(enhancement)
                    }
                }
                4..=6 => {
                    if rng.gen_bool(0.5) {
                        Equipment::iron_sword(enhancement)
                    } else {
                        Equipment::iron_shield(enhancement)
                    }
                }
                _ => {
                    if rng.gen_bool(0.5) {
                        Equipment::steel_sword(enhancement)
                    } else {
                        Equipment::steel_shield(enhancement)
                    }
                }
            };
            self.floor_equips.push(FloorEquipment { equip, pos });
        }

        // Spawn player
        let player_pos = calc_spawn_pos(&self.map, &occupied);
        self.player.pos = player_pos;
    }

    pub fn process_turn(&mut self, command: GameCommand) -> Result<Vec<GameEvent>, TurnError> {
        let mut events = Vec::new();
        let mut turn_consumed = true;

        match command {
            GameCommand::Move(dir) => {
                self.player.direction = dir;
                let target = self.player.pos.plus(&dir.to_offset());

                if let Some(monster_idx) = self.monster_at(&target) {
                    self.attack_monster(monster_idx, &mut events);
                } else if self.map.is_exit(&target) {
                    self.floor += 1;
                    if self.floor > self.max_floor {
                        events.push(GameEvent::GameClear);
                        return Ok(events);
                    } else {
                        events.push(GameEvent::FloorAdvance { new_floor: self.floor });
                        self.map = generator::generate();
                        self.visibility.reset_for_new_floor();
                        self.spawn_entities();
                        self.visibility.update(&self.player.pos, &self.map);
                        return Ok(events);
                    }
                } else if self.map.is_walkable(&target) && self.monster_at(&target).is_none() {
                    let from = self.player.pos;
                    self.player.pos = target;
                    self.visibility.update(&self.player.pos, &self.map);
                    events.push(GameEvent::PlayerMoved { from, to: target });

                    // Check item pickup
                    self.check_pickup(&mut events);
                }
            }
            GameCommand::UseItem(idx) => {
                if idx >= self.player.inventory.len() {
                    return Err(TurnError::InvalidInventoryIndex(idx));
                }
                self.use_item(idx, &mut events);
            }
            GameCommand::UseStaff(idx) => {
                if idx >= self.player.inventory.len() {
                    return Err(TurnError::InvalidInventoryIndex(idx));
                }
                self.use_staff(idx, &mut events);
            }
            GameCommand::EquipWeapon(idx) => {
                if idx >= self.player.inventory.len() {
                    return Err(TurnError::InvalidInventoryIndex(idx));
                }
                let item = self.player.inventory.remove(idx);
                if let Some(old) = self.player.weapon.take() {
                    self.player.inventory.push(equipment_to_item(&old));
                }
                let name = item.name.clone();
                self.player.weapon = Some(item_to_equipment(&item, EquipCategory::Weapon));
                events.push(GameEvent::Equipped { name });
            }
            GameCommand::EquipShield(idx) => {
                if idx >= self.player.inventory.len() {
                    return Err(TurnError::InvalidInventoryIndex(idx));
                }
                let item = self.player.inventory.remove(idx);
                if let Some(old) = self.player.shield.take() {
                    self.player.inventory.push(equipment_to_item(&old));
                }
                let name = item.name.clone();
                self.player.shield = Some(item_to_equipment(&item, EquipCategory::Shield));
                events.push(GameEvent::Equipped { name });
            }
            GameCommand::Wait => {}
            GameCommand::OpenInventory => {
                events.push(GameEvent::RequestInventory);
                turn_consumed = false;
            }
            GameCommand::Quit => return Ok(events),
        }

        if !turn_consumed {
            return Ok(events);
        }

        // Fullness decrease
        self.process_fullness(&mut events);

        // Check game over after player action
        if self.player.hp <= 0 {
            events.push(GameEvent::GameOver);
            return Ok(events);
        }

        // Monster turns
        self.process_monster_turns(&mut events);

        // Check game over after monster actions
        if self.player.hp <= 0 {
            events.push(GameEvent::GameOver);
        }

        // Tick status effects
        self.player.status.tick();
        for m in &mut self.monsters {
            m.status.tick();
        }

        self.turn += 1;
        Ok(events)
    }

    fn attack_monster(&mut self, monster_idx: usize, events: &mut Vec<GameEvent>) {
        let attack = self.player.effective_attack();
        let damage = std::cmp::max(1, attack - self.monsters[monster_idx].defense);
        self.monsters[monster_idx].hp -= damage;
        // Release paralysis on hit
        if self.monsters[monster_idx].status.paralyzed {
            self.monsters[monster_idx].status.paralyzed = false;
            events.push(GameEvent::Message("金縛りが解けた".into()));
        }
        let name = self.monsters[monster_idx].name.clone();
        events.push(GameEvent::PlayerAttacked {
            target_name: name.clone(),
            damage,
        });

        if self.monsters[monster_idx].hp <= 0 {
            let exp = self.monsters[monster_idx].exp;
            self.monsters.remove(monster_idx);
            events.push(GameEvent::MonsterDefeated {
                name: name.clone(),
                exp,
            });
            self.player.exp += exp;
            self.player.kill_count += 1;
            self.check_level_up(events);
        }
    }

    fn check_pickup(&mut self, events: &mut Vec<GameEvent>) {
        let pos = self.player.pos;

        // Pick up items
        if let Some(idx) = self.floor_items.iter().position(|fi| fi.pos == pos) {
            if self.player.inventory_full() {
                events.push(GameEvent::InventoryFull);
            } else {
                let fi = self.floor_items.remove(idx);
                let name = fi.item.name.clone();
                self.player.inventory.push(fi.item);
                events.push(GameEvent::ItemPickedUp { name });
            }
        }

        // Pick up equipment
        if let Some(idx) = self.floor_equips.iter().position(|fe| fe.pos == pos) {
            if self.player.inventory_full() {
                events.push(GameEvent::InventoryFull);
            } else {
                let fe = self.floor_equips.remove(idx);
                let name = fe.equip.display_name();
                let base_value = fe.equip.base_value;
                let enhancement = fe.equip.enhancement;
                let item = Item {
                    id: fe.equip.id,
                    name: name.clone(),
                    symbol: fe.equip.symbol,
                    category: match fe.equip.category {
                        EquipCategory::Weapon => ItemCategory::Weapon,
                        EquipCategory::Shield => ItemCategory::Shield,
                    },
                    effect: ItemEffect::Heal(0),
                    equip_data: Some(EquipData { base_value, enhancement }),
                };
                self.player.inventory.push(item);
                events.push(GameEvent::EquipmentPickedUp { name });
            }
        }
    }

    fn use_item(&mut self, idx: usize, events: &mut Vec<GameEvent>) {
        let item = &self.player.inventory[idx];
        let name = item.name.clone();
        let effect_desc;

        match &item.effect {
            ItemEffect::Heal(amount) => {
                if matches!(item.category, ItemCategory::Weapon | ItemCategory::Shield) {
                    // Can't "use" equipment
                    events.push(GameEvent::Message("装備品は使えない".into()));
                    return;
                }
                let healed = std::cmp::min(*amount, self.player.max_hp - self.player.hp);
                self.player.hp += healed;
                if healed > 0 {
                    effect_desc = format!("HPが{}回復した", healed);
                } else {
                    // At max HP, boost max_hp by 1
                    self.player.max_hp += 1;
                    self.player.hp += 1;
                    effect_desc = "最大HPが1上がった".into();
                }
            }
            ItemEffect::HealFull => {
                let healed = self.player.max_hp - self.player.hp;
                self.player.hp = self.player.max_hp;
                effect_desc = format!("HPが{}回復した", healed);
            }
            ItemEffect::Food(amount) => {
                let recovered = std::cmp::min(*amount, self.player.max_fullness - self.player.fullness);
                self.player.fullness = std::cmp::min(self.player.fullness + amount, self.player.max_fullness);
                effect_desc = format!("満腹度が{}回復した", recovered / 10);
            }
            ItemEffect::BoostAttack(amount) => {
                self.player.attack += amount;
                effect_desc = format!("攻撃力が{}上がった", amount);
            }
            ItemEffect::RevealMap => {
                self.visibility.full_map = true;
                self.visibility.update(&self.player.pos, &self.map);
                effect_desc = "フロア全体が明るくなった".into();
            }
            ItemEffect::ConfuseAll { turns } => {
                let count = self.monsters.len();
                for m in &mut self.monsters {
                    m.status.confused = Some(*turns);
                }
                effect_desc = format!("フロアのモンスター{}体が混乱した", count);
            }
            ItemEffect::TempBoostAttack { amount, turns } => {
                self.player.status.attack_boost = Some((*amount, *turns));
                effect_desc = format!("攻撃力が{}ターンの間{}上がった", turns, amount);
            }
            _ => {
                events.push(GameEvent::Message("このアイテムは使えない".into()));
                return;
            }
        }

        self.player.inventory.remove(idx);
        events.push(GameEvent::ItemUsed { name, effect_desc });
    }

    fn use_staff(&mut self, idx: usize, events: &mut Vec<GameEvent>) {
        let item = &self.player.inventory[idx];
        let name = item.name.clone();
        let effect = item.effect.clone();

        // Check charges
        let charges = match &item.category {
            ItemCategory::Staff(c) => *c,
            _ => {
                events.push(GameEvent::Message("杖ではない".into()));
                return;
            }
        };

        if charges <= 0 {
            events.push(GameEvent::Message(format!("{}を振ったが何も起こらなかった", name)));
            return;
        }

        // Decrease charges
        if let ItemCategory::Staff(c) = &mut self.player.inventory[idx].category {
            *c -= 1;
        }

        // Fire magic bolt in player's direction
        let dir_offset = self.player.direction.to_offset();
        let mut bolt_pos = self.player.pos;
        let mut hit_monster: Option<usize> = None;

        for _ in 0..20 {
            bolt_pos = bolt_pos.plus(&dir_offset);
            if !self.map.is_walkable(&bolt_pos) {
                break;
            }
            if let Some(mi) = self.monster_at(&bolt_pos) {
                hit_monster = Some(mi);
                break;
            }
        }

        match hit_monster {
            None => {
                events.push(GameEvent::Message(format!("{}を振った。魔法弾は何にも当たらなかった", name)));
            }
            Some(mi) => {
                let monster_name = self.monsters[mi].name.clone();
                match &effect {
                    ItemEffect::Paralyze => {
                        self.monsters[mi].status.paralyzed = true;
                        events.push(GameEvent::Message(format!("{}は金縛りになった", monster_name)));
                    }
                    ItemEffect::Knockback { distance } => {
                        let knocked = self.knockback_monster(mi, &dir_offset, *distance);
                        events.push(GameEvent::Message(format!("{}を{}マス吹き飛ばした", monster_name, knocked)));
                    }
                    ItemEffect::SwapPosition => {
                        let player_pos = self.player.pos;
                        let monster_pos = self.monsters[mi].pos;
                        self.player.pos = monster_pos;
                        self.monsters[mi].pos = player_pos;
                        self.visibility.update(&self.player.pos, &self.map);
                        events.push(GameEvent::Message(format!("{}と場所を入れ替えた", monster_name)));
                    }
                    _ => {}
                }
            }
        }
    }

    fn knockback_monster(&mut self, monster_idx: usize, dir: &Position, max_distance: i32) -> i32 {
        let mut moved = 0;
        for _ in 0..max_distance {
            let next = self.monsters[monster_idx].pos.plus(dir);
            if !self.map.is_walkable(&next) || self.monster_at(&next).is_some() || next == self.player.pos {
                break;
            }
            self.monsters[monster_idx].pos = next;
            moved += 1;
        }
        moved
    }

    fn process_fullness(&mut self, events: &mut Vec<GameEvent>) {
        if self.player.fullness > 0 {
            self.player.fullness -= 1;
        } else {
            self.player.hp -= 1;
            events.push(GameEvent::Starving);
        }
    }

    fn process_monster_turns(&mut self, events: &mut Vec<GameEvent>) {
        let mut rng = thread_rng();
        let defense = self.player.effective_defense();

        for i in 0..self.monsters.len() {
            // Paralyzed monsters skip their turn entirely
            if self.monsters[i].status.paralyzed {
                continue;
            }

            let monster_pos = self.monsters[i].pos;

            // Confused monsters move randomly (no attacking)
            if self.monsters[i].status.confused.is_some() {
                let dirs: Vec<&Direction> = Direction::all()
                    .iter()
                    .filter(|d| {
                        let p = monster_pos.plus(&d.to_offset());
                        self.map.is_walkable(&p)
                            && self.monster_at(&p).is_none()
                            && p != self.player.pos
                    })
                    .collect();
                if !dirs.is_empty() {
                    let dir = dirs[rng.gen_range(0..dirs.len())];
                    let from = self.monsters[i].pos;
                    let new_pos = monster_pos.plus(&dir.to_offset());
                    self.monsters[i].pos = new_pos;
                    events.push(GameEvent::MonsterMoved { id: i, from, to: new_pos });
                }
                continue;
            }

            // Check if adjacent to player -> attack
            if (monster_pos.x - self.player.pos.x).abs() <= 1
                && (monster_pos.y - self.player.pos.y).abs() <= 1
                && monster_pos != self.player.pos
            {
                let damage = std::cmp::max(1, self.monsters[i].attack - defense);
                self.player.hp -= damage;
                let name = self.monsters[i].name.clone();
                events.push(GameEvent::MonsterAttacked {
                    name: name.clone(),
                    damage,
                });
                events.push(GameEvent::PlayerDamaged {
                    amount: damage,
                    remaining_hp: self.player.hp,
                });
                if self.player.hp <= 0 {
                    return;
                }
                continue;
            }

            // Check if player is in same room -> chase
            if self.map.is_same_room(&monster_pos, &self.player.pos) {
                if let Some(new_pos) = self.chase_player(i) {
                    let from = self.monsters[i].pos;
                    self.monsters[i].pos = new_pos;
                    events.push(GameEvent::MonsterMoved { id: i, from, to: new_pos });
                    continue;
                }
            }

            // Check corridor detection range
            if monster_pos.manhattan_distance(&self.player.pos) <= 5 {
                if let Some(new_pos) = self.chase_player(i) {
                    let from = self.monsters[i].pos;
                    self.monsters[i].pos = new_pos;
                    events.push(GameEvent::MonsterMoved { id: i, from, to: new_pos });
                    continue;
                }
            }

            // Random movement
            let dirs: Vec<&Direction> = Direction::all()
                .iter()
                .filter(|d| {
                    let p = monster_pos.plus(&d.to_offset());
                    self.map.is_walkable(&p)
                        && self.monster_at(&p).is_none()
                        && p != self.player.pos
                })
                .collect();

            if !dirs.is_empty() {
                let dir = dirs[rng.gen_range(0..dirs.len())];
                let from = self.monsters[i].pos;
                let new_pos = monster_pos.plus(&dir.to_offset());
                self.monsters[i].pos = new_pos;
                events.push(GameEvent::MonsterMoved { id: i, from, to: new_pos });
            }
        }
    }

    fn chase_player(&self, monster_idx: usize) -> Option<Position> {
        let mpos = self.monsters[monster_idx].pos;
        let ppos = self.player.pos;

        let dx = (ppos.x - mpos.x).signum();
        let dy = (ppos.y - mpos.y).signum();

        let candidates = [
            Position::new(dx, dy),
            Position::new(dx, 0),
            Position::new(0, dy),
        ];

        for offset in &candidates {
            if offset.x == 0 && offset.y == 0 {
                continue;
            }
            let target = mpos.plus(offset);
            if (target.x - ppos.x).abs() <= 1 && (target.y - ppos.y).abs() <= 1 && target != ppos {
                // Will be adjacent, stop to attack next turn
                return Some(target);
            }
            if target == ppos {
                return None;
            }
            if self.map.is_walkable(&target) && self.monster_at(&target).is_none() {
                return Some(target);
            }
        }

        None
    }

    fn monster_at(&self, pos: &Position) -> Option<usize> {
        self.monsters.iter().position(|m| m.pos == *pos)
    }

    fn check_level_up(&mut self, events: &mut Vec<GameEvent>) {
        let level_table: Vec<(i32, i32, i32, i32, i32)> = vec![
            (0, 30, 8, 5, 1),
            (30, 35, 10, 6, 2),
            (70, 40, 12, 7, 3),
            (120, 45, 14, 8, 4),
            (200, 50, 16, 9, 5),
        ];

        for &(req_exp, max_hp, attack, defense, level) in level_table.iter().rev() {
            if self.player.exp >= req_exp && self.player.level < level {
                self.player.level = level;
                self.player.max_hp = max_hp;
                self.player.hp = max_hp;
                self.player.attack = attack;
                self.player.defense = defense;
                events.push(GameEvent::LevelUp { new_level: level });
                break;
            }
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.player.hp <= 0
    }

    pub fn is_game_clear(&self) -> bool {
        self.floor > self.max_floor
    }
}

fn equipment_to_item(equip: &Equipment) -> Item {
    Item {
        id: equip.id.clone(),
        name: equip.display_name(),
        symbol: equip.symbol,
        category: match equip.category {
            EquipCategory::Weapon => ItemCategory::Weapon,
            EquipCategory::Shield => ItemCategory::Shield,
        },
        effect: ItemEffect::Heal(0),
        equip_data: Some(EquipData {
            base_value: equip.base_value,
            enhancement: equip.enhancement,
        }),
    }
}

fn item_to_equipment(item: &Item, category: EquipCategory) -> Equipment {
    let (base_value, enhancement) = match &item.equip_data {
        Some(data) => (data.base_value, data.enhancement),
        None => (0, 0),
    };
    Equipment {
        id: item.id.clone(),
        name: item.name.clone(),
        symbol: item.symbol,
        category,
        base_value,
        enhancement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::cell::{MapCell, Terrain};

    /// Create a controlled 10x10 GameState with a single room and no randomness
    fn make_test_state() -> GameState {
        let mut map = GameMap::new(10, 10, Position::new(8, 8));
        // Room at (1,1)-(8,8)
        for y in 1..=8 {
            for x in 1..=8 {
                map.set(x, y, MapCell { terrain: Terrain::Floor { room_id: 1 } });
            }
        }
        map.set(8, 8, MapCell { terrain: Terrain::Exit });

        let mut player = Player::new();
        player.pos = Position::new(2, 2);

        let mut state = GameState {
            player,
            monsters: Vec::new(),
            floor_items: Vec::new(),
            floor_equips: Vec::new(),
            map,
            visibility: Visibility::new(),
            floor: 1,
            max_floor: 10,
            turn: 0,
        };
        state.visibility.update(&state.player.pos, &state.map);
        state
    }

    // --- Movement ---

    #[test]
    fn move_to_walkable_tile() {
        let mut state = make_test_state();
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert_eq!(state.player.pos, Position::new(3, 2));
        assert!(events.iter().any(|e| matches!(e, GameEvent::PlayerMoved { .. })));
    }

    #[test]
    fn move_into_wall_stays() {
        let mut state = make_test_state();
        state.player.pos = Position::new(1, 1);
        state.process_turn(GameCommand::Move(Direction::Left)).unwrap();
        assert_eq!(state.player.pos, Position::new(1, 1)); // blocked by wall
    }

    #[test]
    fn move_onto_exit_advances_floor() {
        let mut state = make_test_state();
        state.player.pos = Position::new(7, 8);
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert_eq!(state.floor, 2);
        assert!(events.iter().any(|e| matches!(e, GameEvent::FloorAdvance { .. })));
    }

    #[test]
    fn move_onto_exit_floor_10_game_clear() {
        let mut state = make_test_state();
        state.floor = 10;
        state.max_floor = 10;
        state.player.pos = Position::new(7, 8);
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::GameClear)));
    }

    // --- Combat ---

    #[test]
    fn attack_monster_deals_damage() {
        let mut state = make_test_state();
        let def = &monsters_for_floor(1)[0]; // slime: hp=5, def=3
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));

        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        // Player attack=8, monster defense=3, damage=max(1, 8-3)=5
        assert!(events.iter().any(|e| matches!(e, GameEvent::PlayerAttacked { damage: 5, .. })));
        // Slime has 5 hp, should be defeated
        assert!(events.iter().any(|e| matches!(e, GameEvent::MonsterDefeated { .. })));
        assert!(state.monsters.is_empty());
    }

    #[test]
    fn attack_monster_gains_exp() {
        let mut state = make_test_state();
        let def = &monsters_for_floor(1)[0]; // slime: exp=4
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));
        state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert_eq!(state.player.exp, 4);
    }

    #[test]
    fn attack_releases_paralysis() {
        let mut state = make_test_state();
        let def = &monsters_for_floor(4)[2]; // golem: hp=20
        let mut monster = Monster::from_def(def, Position::new(3, 2));
        monster.status.paralyzed = true;
        state.monsters.push(monster);

        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert!(!state.monsters[0].status.paralyzed);
        assert!(events.iter().any(|e| matches!(e, GameEvent::Message(m) if m.contains("金縛り"))));
    }

    // --- Item pickup ---

    #[test]
    fn item_pickup_on_move() {
        let mut state = make_test_state();
        state.floor_items.push(FloorItem {
            item: Item::herb(),
            pos: Position::new(3, 2),
        });
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::ItemPickedUp { .. })));
        assert_eq!(state.player.inventory.len(), 1);
        assert!(state.floor_items.is_empty());
    }

    #[test]
    fn item_pickup_inventory_full() {
        let mut state = make_test_state();
        for _ in 0..MAX_INVENTORY {
            state.player.inventory.push(Item::herb());
        }
        state.floor_items.push(FloorItem {
            item: Item::herb(),
            pos: Position::new(3, 2),
        });
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::InventoryFull)));
        assert_eq!(state.floor_items.len(), 1); // item stays on floor
    }

    // --- Item usage ---

    #[test]
    fn use_herb_heals() {
        let mut state = make_test_state();
        state.player.hp = 10;
        state.player.inventory.push(Item::herb()); // heals 25
        let events = state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.hp, 30); // min(10+25, 30)
        assert!(events.iter().any(|e| matches!(e, GameEvent::ItemUsed { .. })));
        assert!(state.player.inventory.is_empty());
    }

    #[test]
    fn use_herb_at_max_hp_boosts_max() {
        let mut state = make_test_state();
        // hp == max_hp
        state.player.inventory.push(Item::herb());
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.max_hp, 31);
        assert_eq!(state.player.hp, 31);
    }

    #[test]
    fn use_food_restores_fullness() {
        let mut state = make_test_state();
        state.player.fullness = 500;
        state.player.inventory.push(Item::ration()); // Food(500)
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        // 500 + 500 = 1000, but process_fullness decremented by 1 during turn
        assert_eq!(state.player.fullness, 999);
    }

    #[test]
    fn use_power_herb_boosts_attack() {
        let mut state = make_test_state();
        let old_attack = state.player.attack;
        state.player.inventory.push(Item::power_herb()); // BoostAttack(1)
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.attack, old_attack + 1);
    }

    // --- Fullness / Starvation ---

    #[test]
    fn fullness_decreases_each_turn() {
        let mut state = make_test_state();
        let initial = state.player.fullness;
        state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.player.fullness, initial - 1);
    }

    #[test]
    fn starvation_damages_player() {
        let mut state = make_test_state();
        state.player.fullness = 0;
        let hp_before = state.player.hp;
        let events = state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.player.hp, hp_before - 1);
        assert!(events.iter().any(|e| matches!(e, GameEvent::Starving)));
    }

    // --- Level up ---

    #[test]
    fn level_up_on_exp_threshold() {
        let mut state = make_test_state();
        state.player.exp = 29; // threshold is 30 for level 2
        // Give a monster that yields 4 exp (slime)
        let def = &monsters_for_floor(1)[0];
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));
        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        // 29 + 4 = 33 >= 30 -> level 2
        assert_eq!(state.player.level, 2);
        assert!(events.iter().any(|e| matches!(e, GameEvent::LevelUp { new_level: 2 })));
    }

    // --- Equipment ---

    #[test]
    fn equip_weapon_from_inventory() {
        let mut state = make_test_state();
        let sword = Equipment::iron_sword(1);
        let item = equipment_to_item(&sword);
        state.player.inventory.push(item);

        let events = state.process_turn(GameCommand::EquipWeapon(0)).unwrap();
        assert!(state.player.weapon.is_some());
        assert_eq!(state.player.weapon.as_ref().unwrap().base_value, 6);
        assert!(events.iter().any(|e| matches!(e, GameEvent::Equipped { .. })));
        assert!(state.player.inventory.is_empty());
    }

    #[test]
    fn equip_weapon_swaps_old_to_inventory() {
        let mut state = make_test_state();
        state.player.weapon = Some(Equipment::wooden_sword(0));
        let new_sword = equipment_to_item(&Equipment::iron_sword(0));
        state.player.inventory.push(new_sword);

        state.process_turn(GameCommand::EquipWeapon(0)).unwrap();
        assert_eq!(state.player.weapon.as_ref().unwrap().base_value, 6); // iron
        assert_eq!(state.player.inventory.len(), 1); // old wooden sword returned
    }

    // --- Staff usage ---

    #[test]
    fn use_paralysis_staff_paralyzes_monster() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(Item::paralysis_staff(3));
        // Monster in line of fire
        let def = &monsters_for_floor(4)[2]; // golem
        state.monsters.push(Monster::from_def(def, Position::new(5, 2)));

        state.process_turn(GameCommand::UseStaff(0)).unwrap();
        assert!(state.monsters[0].status.paralyzed);
    }

    #[test]
    fn use_staff_decreases_charges() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(Item::paralysis_staff(3));
        state.monsters.push(Monster::from_def(
            &monsters_for_floor(4)[2],
            Position::new(5, 2),
        ));

        state.process_turn(GameCommand::UseStaff(0)).unwrap();
        match &state.player.inventory[0].category {
            ItemCategory::Staff(c) => assert_eq!(*c, 2),
            _ => panic!("Expected staff"),
        }
    }

    #[test]
    fn use_swap_staff_swaps_positions() {
        let mut state = make_test_state();
        state.player.pos = Position::new(2, 2);
        state.player.direction = Direction::Right;
        state.player.inventory.push(Item::swap_staff(3));
        let def = &monsters_for_floor(4)[2]; // golem
        let mut monster = Monster::from_def(def, Position::new(5, 2));
        // Paralyze so it doesn't move after swap
        monster.status.paralyzed = true;
        state.monsters.push(monster);

        state.process_turn(GameCommand::UseStaff(0)).unwrap();
        assert_eq!(state.player.pos, Position::new(5, 2));
        assert_eq!(state.monsters[0].pos, Position::new(2, 2));
    }

    // --- Monster turns ---

    #[test]
    fn paralyzed_monster_does_not_act() {
        let mut state = make_test_state();
        let def = &monsters_for_floor(1)[0];
        let mut monster = Monster::from_def(def, Position::new(3, 2)); // adjacent
        monster.status.paralyzed = true;
        let pos_before = monster.pos;
        state.monsters.push(monster);

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
        // Paralyzed monster should not attack or move
        assert_eq!(state.player.hp, hp_before);
        assert_eq!(state.monsters[0].pos, pos_before);
    }

    // --- Game state checks ---

    #[test]
    fn is_game_over() {
        let mut state = make_test_state();
        assert!(!state.is_game_over());
        state.player.hp = 0;
        assert!(state.is_game_over());
    }

    #[test]
    fn is_game_clear() {
        let mut state = make_test_state();
        assert!(!state.is_game_clear());
        state.floor = 11;
        state.max_floor = 10;
        assert!(state.is_game_clear());
    }

    #[test]
    fn wait_increments_turn() {
        let mut state = make_test_state();
        assert_eq!(state.turn, 0);
        state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.turn, 1);
    }

    #[test]
    fn open_inventory_does_not_consume_turn() {
        let mut state = make_test_state();
        let events = state.process_turn(GameCommand::OpenInventory).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::RequestInventory)));
        assert_eq!(state.turn, 0); // turn not consumed
    }

    // --- Monster count per floor ---

    #[test]
    fn monster_count_for_floor_values() {
        assert_eq!(GameState::monster_count_for_floor(1), 8);
        assert_eq!(GameState::monster_count_for_floor(3), 8);
        assert_eq!(GameState::monster_count_for_floor(4), 10);
        assert_eq!(GameState::monster_count_for_floor(7), 12);
        assert_eq!(GameState::monster_count_for_floor(10), 15);
    }

    // --- Confused monster ---

    #[test]
    fn confused_monster_does_not_attack() {
        let mut state = make_test_state();
        let def = &monsters_for_floor(1)[0]; // slime
        let mut monster = Monster::from_def(def, Position::new(3, 2)); // adjacent
        monster.status.confused = Some(5);
        state.monsters.push(monster);

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
        // Confused monster moves randomly but does not attack
        assert_eq!(state.player.hp, hp_before);
    }

    // === 境界条件テスト ===

    // #1: ダメージ最低保証値（プレイヤー→モンスター）
    #[test]
    fn attack_damage_clamped_to_one_when_defense_exceeds_attack() {
        let mut state = make_test_state();
        state.player.attack = 1;
        state.player.weapon = None;
        // guardian: hp=60, defense=30
        let def = &monsters_for_floor(10)[2];
        assert_eq!(def.id, "guardian");
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));

        let events = state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        // effective_attack=1, defense=30 → max(1, 1-30) = 1
        assert!(events.iter().any(|e| matches!(e, GameEvent::PlayerAttacked { damage: 1, .. })));
        assert_eq!(state.monsters[0].hp, 60 - 1);
    }

    // #2: ダメージ最低保証値（モンスター→プレイヤー）
    #[test]
    fn monster_damage_clamped_to_one_when_defense_exceeds_attack() {
        let mut state = make_test_state();
        // Give player high defense via shield
        state.player.shield = Some(Equipment::steel_shield(0)); // base=9
        // effective_defense = 5 + 9 = 14
        // slime: attack=2 → max(1, 2-14) = 1
        let def = &monsters_for_floor(1)[0]; // slime
        state.monsters.push(Monster::from_def(def, Position::new(3, 2))); // adjacent

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.player.hp, hp_before - 1);
        assert!(state.player.hp > 0);
    }

    // #5: 回復草のHP上限クランプ
    #[test]
    fn use_herb_clamps_to_max_hp() {
        let mut state = make_test_state();
        state.player.hp = 29; // max_hp=30, heal=25 → only 1 healed
        state.player.inventory.push(Item::herb());
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.hp, 30);
        assert_eq!(state.player.max_hp, 30); // max_hp should NOT increase
    }

    // #6: 満腹度が上限の状態でFood使用
    #[test]
    fn use_food_at_max_fullness() {
        let mut state = make_test_state();
        state.player.fullness = 1000; // already max
        state.player.max_fullness = 1000;
        state.player.inventory.push(Item::ration()); // Food(500)
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        // process_fullness decrements by 1, food adds 0 (clamped)
        assert_eq!(state.player.fullness, 999);
    }

    // #7: チャージ0の杖を使用
    #[test]
    fn use_staff_with_zero_charges() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(Item::paralysis_staff(0)); // 0 charges
        let def = &monsters_for_floor(4)[2]; // golem
        state.monsters.push(Monster::from_def(def, Position::new(5, 2)));

        let events = state.process_turn(GameCommand::UseStaff(0)).unwrap();
        // Should get "nothing happened" message
        assert!(events.iter().any(|e| matches!(e, GameEvent::Message(m) if m.contains("何も起こらなかった"))));
        // Monster should NOT be paralyzed
        assert!(!state.monsters[0].status.paralyzed);
    }

    // #8: インベントリ範囲外インデックス
    #[test]
    fn use_item_out_of_bounds_index() {
        let mut state = make_test_state();
        let result = state.process_turn(GameCommand::UseItem(99));
        assert!(matches!(result, Err(TurnError::InvalidInventoryIndex(99))));
        assert_eq!(state.turn, 0); // turn not consumed
    }

    #[test]
    fn use_staff_out_of_bounds_index() {
        let mut state = make_test_state();
        let result = state.process_turn(GameCommand::UseStaff(99));
        assert!(matches!(result, Err(TurnError::InvalidInventoryIndex(99))));
        assert_eq!(state.turn, 0);
    }

    #[test]
    fn equip_weapon_out_of_bounds_index() {
        let mut state = make_test_state();
        let result = state.process_turn(GameCommand::EquipWeapon(99));
        assert!(matches!(result, Err(TurnError::InvalidInventoryIndex(99))));
        assert!(state.player.weapon.is_none());
        assert_eq!(state.turn, 0);
    }

    // #9: レベルアップ閾値ぴったり
    #[test]
    fn level_up_at_exact_threshold() {
        let mut state = make_test_state();
        state.player.exp = 30; // exactly level 2 threshold
        // Need to trigger check_level_up — kill a monster to re-check
        let def = &monsters_for_floor(1)[0]; // slime: exp=4
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));
        state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        // 30 + 4 = 34 >= 30 → level 2
        assert_eq!(state.player.level, 2);
    }

    #[test]
    fn no_level_up_below_threshold() {
        let mut state = make_test_state();
        state.player.exp = 25;
        let def = &monsters_for_floor(1)[0]; // slime: exp=4
        state.monsters.push(Monster::from_def(def, Position::new(3, 2)));
        state.process_turn(GameCommand::Move(Direction::Right)).unwrap();
        // 25 + 4 = 29 < 30 → still level 1
        assert_eq!(state.player.level, 1);
    }

    // #11: 餓死→GameOver
    #[test]
    fn starvation_causes_game_over() {
        let mut state = make_test_state();
        state.player.fullness = 0;
        state.player.hp = 1;
        let events = state.process_turn(GameCommand::Wait).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::Starving)));
        assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver)));
        assert_eq!(state.player.hp, 0);
    }
}
