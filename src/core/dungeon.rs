use std::collections::HashSet;
use std::sync::Arc;

use rand::prelude::*;

use super::entity::*;
use super::master_data::MasterData;
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
    pub turn: u32,
    pub data: Arc<MasterData>,
}

impl GameState {
    pub fn new(data: Arc<MasterData>) -> Self {
        let map = generator::generate(&data.map);
        let mut state = GameState {
            player: Player::new(&data.player),
            monsters: Vec::new(),
            floor_items: Vec::new(),
            floor_equips: Vec::new(),
            map,
            visibility: Visibility::new(),
            floor: 1,
            turn: 0,
            data,
        };
        state.spawn_entities();
        state.visibility.update(&state.player.pos, &state.map);
        state
    }

    fn monster_count_for_floor(&self, floor: u32) -> u32 {
        for entry in &self.data.floors.monster_counts {
            if entry.floors.contains(floor) {
                return entry.count;
            }
        }
        // Fallback: use last entry's count
        self.data
            .floors
            .monster_counts
            .last()
            .map_or(8, |e| e.count)
    }

    fn monsters_for_floor(&self, floor: u32) -> Vec<String> {
        for entry in &self.data.floors.monster_table {
            if entry.floors.contains(floor) {
                return entry.monsters.clone();
            }
        }
        // Fallback: use last entry
        self.data
            .floors
            .monster_table
            .last()
            .map(|e| e.monsters.clone())
            .unwrap_or_default()
    }

    fn spawn_entities(&mut self) {
        let monster_count = self.monster_count_for_floor(self.floor);
        let mut occupied = HashSet::new();

        // Spawn monsters from floor table
        self.monsters.clear();
        let monster_ids = self.monsters_for_floor(self.floor);
        let mut rng = thread_rng();
        for _ in 0..monster_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let id = &monster_ids[rng.gen_range(0..monster_ids.len())];
            if let Some(def) = self.data.monsters.get(id.as_str()) {
                self.monsters.push(Monster::from_stats_def(def, pos));
            }
        }

        // Spawn herbs
        self.floor_items.clear();
        let herb_count = self
            .data
            .floors
            .herb_spawns
            .iter()
            .find(|e| e.floors.contains(self.floor))
            .map_or(3, |e| e.count);
        let herb_ids: Vec<&str> = self
            .data
            .items
            .iter()
            .filter(|(_, def)| matches!(def.category, super::master_data::ItemCategoryDef::Herb))
            .map(|(id, _)| id.as_str())
            .collect();
        if !herb_ids.is_empty() {
            for _ in 0..herb_count {
                let pos = calc_spawn_pos(&self.map, &occupied);
                occupied.insert(pos);
                let id = herb_ids[rng.gen_range(0..herb_ids.len())];
                if let Some(def) = self.data.items.get(id) {
                    self.floor_items.push(FloorItem {
                        item: Item::from_def(id, def, None),
                        pos,
                    });
                }
            }
        }

        // Spawn food
        for food_entry in &self.data.floors.food_spawns {
            if food_entry.floors.contains(self.floor) {
                for _ in 0..food_entry.count {
                    let pos = calc_spawn_pos(&self.map, &occupied);
                    occupied.insert(pos);
                    // Weighted random selection
                    let total_weight: f64 = food_entry.items.iter().map(|i| i.weight).sum();
                    let mut roll = rng.gen::<f64>() * total_weight;
                    let mut chosen_id = &food_entry.items[0].id;
                    for fi in &food_entry.items {
                        roll -= fi.weight;
                        if roll <= 0.0 {
                            chosen_id = &fi.id;
                            break;
                        }
                    }
                    if let Some(def) = self.data.items.get(chosen_id.as_str()) {
                        self.floor_items.push(FloorItem {
                            item: Item::from_def(chosen_id, def, None),
                            pos,
                        });
                    }
                }
                break;
            }
        }

        // Spawn scrolls
        let scroll_spawns = &self.data.floors.scroll_spawns;
        let scroll_count = rng.gen_range(scroll_spawns.min..=scroll_spawns.max);
        for _ in 0..scroll_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let id = &scroll_spawns.pool[rng.gen_range(0..scroll_spawns.pool.len())];
            if let Some(def) = self.data.items.get(id.as_str()) {
                self.floor_items.push(FloorItem {
                    item: Item::from_def(id, def, None),
                    pos,
                });
            }
        }

        // Spawn staffs
        let staff_spawns = &self.data.floors.staff_spawns;
        if rng.gen_bool(staff_spawns.chance) {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            let charges = rng.gen_range(staff_spawns.min_charges..=staff_spawns.max_charges);
            let id = &staff_spawns.pool[rng.gen_range(0..staff_spawns.pool.len())];
            if let Some(def) = self.data.items.get(id.as_str()) {
                self.floor_items.push(FloorItem {
                    item: Item::from_def(id, def, Some(charges)),
                    pos,
                });
            }
        }

        // Spawn equipment
        self.floor_equips.clear();
        for equip_entry in &self.data.floors.equipment_spawns {
            if equip_entry.floors.contains(self.floor) {
                if rng.gen_bool(equip_entry.chance) {
                    let pos = calc_spawn_pos(&self.map, &occupied);
                    occupied.insert(pos);
                    let enhancement = rng.gen_range(0..=equip_entry.max_enhancement);
                    if rng.gen_bool(0.5) {
                        // Weapon
                        let id = &equip_entry.weapons[rng.gen_range(0..equip_entry.weapons.len())];
                        if let Some(def) = self.data.equipment.get(id.as_str()) {
                            self.floor_equips.push(FloorEquipment {
                                equip: Equipment::from_def(id, def, enhancement),
                                pos,
                            });
                        }
                    } else {
                        // Shield
                        let id = &equip_entry.shields[rng.gen_range(0..equip_entry.shields.len())];
                        if let Some(def) = self.data.equipment.get(id.as_str()) {
                            self.floor_equips.push(FloorEquipment {
                                equip: Equipment::from_def(id, def, enhancement),
                                pos,
                            });
                        }
                    }
                }
                break;
            }
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
                    if self.floor > self.data.map.max_floor {
                        events.push(GameEvent::GameClear);
                        return Ok(events);
                    } else {
                        events.push(GameEvent::FloorAdvance {
                            new_floor: self.floor,
                        });
                        self.map = generator::generate(&self.data.map);
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
            GameCommand::ThrowItem(idx) => {
                if idx >= self.player.inventory.len() {
                    return Err(TurnError::InvalidInventoryIndex(idx));
                }
                self.throw_item(idx, &mut events);
            }
            GameCommand::Wait => {}
            GameCommand::OpenInventory => {
                events.push(GameEvent::RequestInventory);
                turn_consumed = false;
            }
            GameCommand::OpenThrowInventory => {
                events.push(GameEvent::RequestThrowInventory);
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
        let damage = std::cmp::max(
            self.data.balance.min_damage,
            attack - self.monsters[monster_idx].defense,
        );
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
        let max_inv = self.data.player.max_inventory;

        // Pick up items
        if let Some(idx) = self.floor_items.iter().position(|fi| fi.pos == pos) {
            if self.player.inventory_full(max_inv) {
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
            if self.player.inventory_full(max_inv) {
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
                    equip_data: Some(EquipData {
                        base_value,
                        enhancement,
                    }),
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
                let recovered =
                    std::cmp::min(*amount, self.player.max_fullness - self.player.fullness);
                self.player.fullness =
                    std::cmp::min(self.player.fullness + amount, self.player.max_fullness);
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
            events.push(GameEvent::Message(format!(
                "{}を振ったが何も起こらなかった",
                name
            )));
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

        let staff_range = self.data.balance.staff_range;
        for _ in 0..staff_range {
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
                events.push(GameEvent::Message(format!(
                    "{}を振った。魔法弾は何にも当たらなかった",
                    name
                )));
            }
            Some(mi) => {
                let monster_name = self.monsters[mi].name.clone();
                match &effect {
                    ItemEffect::Paralyze => {
                        self.monsters[mi].status.paralyzed = true;
                        events.push(GameEvent::Message(format!(
                            "{}は金縛りになった",
                            monster_name
                        )));
                    }
                    ItemEffect::Knockback { distance } => {
                        let knocked = self.knockback_monster(mi, &dir_offset, *distance);
                        events.push(GameEvent::Message(format!(
                            "{}を{}マス吹き飛ばした",
                            monster_name, knocked
                        )));
                    }
                    ItemEffect::SwapPosition => {
                        let player_pos = self.player.pos;
                        let monster_pos = self.monsters[mi].pos;
                        self.player.pos = monster_pos;
                        self.monsters[mi].pos = player_pos;
                        self.visibility.update(&self.player.pos, &self.map);
                        events.push(GameEvent::Message(format!(
                            "{}と場所を入れ替えた",
                            monster_name
                        )));
                    }
                    _ => {}
                }
            }
        }
    }

    fn throw_item(&mut self, idx: usize, events: &mut Vec<GameEvent>) {
        let item = self.player.inventory.remove(idx);
        let name = item.name.clone();
        let category = item.category.clone();
        let effect = item.effect.clone();

        let dir_offset = self.player.direction.to_offset();
        let mut throw_pos = self.player.pos;
        let mut last_walkable = self.player.pos;

        let throw_range = self.data.balance.throw_range;
        let thrown_non_weapon_damage = self.data.balance.thrown_non_weapon_damage;
        let min_damage = self.data.balance.min_damage;

        for _ in 0..throw_range {
            throw_pos = throw_pos.plus(&dir_offset);

            if !self.map.is_walkable(&throw_pos) {
                // Hit a wall — item is lost
                events.push(GameEvent::ItemThrown {
                    name,
                    result_desc: "壁に当たって砕けた".into(),
                });
                return;
            }

            if let Some(mi) = self.monster_at(&throw_pos) {
                // Hit a monster — deal damage + effects
                let damage = match &category {
                    ItemCategory::Weapon => {
                        let attack_value = item
                            .equip_data
                            .as_ref()
                            .map(|d| d.base_value + d.enhancement)
                            .unwrap_or(0);
                        std::cmp::max(min_damage, attack_value / 2)
                    }
                    _ => thrown_non_weapon_damage,
                };

                self.monsters[mi].hp -= damage;
                let monster_name = self.monsters[mi].name.clone();
                let mut result_desc = format!("{}に{}ダメージを与えた", monster_name, damage);

                // Apply herb effects to monster
                if matches!(category, ItemCategory::Herb) {
                    match &effect {
                        ItemEffect::Heal(amount) => {
                            let healed = std::cmp::min(
                                *amount,
                                self.monsters[mi].max_hp - self.monsters[mi].hp,
                            );
                            if healed > 0 {
                                self.monsters[mi].hp += healed;
                                result_desc +=
                                    &format!("。{}のHPが{}回復した", monster_name, healed);
                            }
                        }
                        ItemEffect::HealFull => {
                            self.monsters[mi].hp = self.monsters[mi].max_hp;
                            result_desc += &format!("。{}のHPが全回復した", monster_name);
                        }
                        ItemEffect::BoostAttack(amount) => {
                            self.monsters[mi].attack += amount;
                            result_desc +=
                                &format!("。{}の攻撃力が{}上がった", monster_name, amount);
                        }
                        _ => {}
                    }
                }

                // Check if monster is defeated
                if self.monsters[mi].hp <= 0 {
                    let exp = self.monsters[mi].exp;
                    self.monsters.remove(mi);
                    events.push(GameEvent::ItemThrown { name, result_desc });
                    events.push(GameEvent::MonsterDefeated {
                        name: monster_name,
                        exp,
                    });
                    self.player.exp += exp;
                    self.player.kill_count += 1;
                    self.check_level_up(events);
                } else {
                    events.push(GameEvent::ItemThrown { name, result_desc });
                }
                return;
            }

            last_walkable = throw_pos;
        }

        // Didn't hit anything — item drops on ground
        self.floor_items.push(FloorItem {
            item,
            pos: last_walkable,
        });
        events.push(GameEvent::ItemThrown {
            name,
            result_desc: "地面に落ちた".into(),
        });
    }

    fn knockback_monster(&mut self, monster_idx: usize, dir: &Position, max_distance: i32) -> i32 {
        let mut moved = 0;
        for _ in 0..max_distance {
            let next = self.monsters[monster_idx].pos.plus(dir);
            if !self.map.is_walkable(&next)
                || self.monster_at(&next).is_some()
                || next == self.player.pos
            {
                break;
            }
            self.monsters[monster_idx].pos = next;
            moved += 1;
        }
        moved
    }

    fn process_fullness(&mut self, events: &mut Vec<GameEvent>) {
        if self.player.fullness > 0 {
            self.player.fullness -= self.data.balance.fullness_decrease_per_turn;
        } else {
            self.player.hp -= self.data.balance.starvation_damage;
            events.push(GameEvent::Starving);
        }
    }

    fn process_monster_turns(&mut self, events: &mut Vec<GameEvent>) {
        let mut rng = thread_rng();
        let defense = self.player.effective_defense();
        let min_damage = self.data.balance.min_damage;
        let detection_range = self.data.balance.detection_range;

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
                    events.push(GameEvent::MonsterMoved {
                        id: i,
                        from,
                        to: new_pos,
                    });
                }
                continue;
            }

            // Check if adjacent to player -> attack
            if (monster_pos.x - self.player.pos.x).abs() <= 1
                && (monster_pos.y - self.player.pos.y).abs() <= 1
                && monster_pos != self.player.pos
            {
                let damage = std::cmp::max(min_damage, self.monsters[i].attack - defense);
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
                    events.push(GameEvent::MonsterMoved {
                        id: i,
                        from,
                        to: new_pos,
                    });
                    continue;
                }
            }

            // Check corridor detection range
            if monster_pos.manhattan_distance(&self.player.pos) <= detection_range {
                if let Some(new_pos) = self.chase_player(i) {
                    let from = self.monsters[i].pos;
                    self.monsters[i].pos = new_pos;
                    events.push(GameEvent::MonsterMoved {
                        id: i,
                        from,
                        to: new_pos,
                    });
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
                events.push(GameEvent::MonsterMoved {
                    id: i,
                    from,
                    to: new_pos,
                });
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
        for entry in self.data.player.level_table.iter().rev() {
            if self.player.exp >= entry.req_exp && self.player.level < entry.level {
                self.player.level = entry.level;
                self.player.max_hp = entry.max_hp;
                self.player.hp = entry.max_hp;
                self.player.attack = entry.attack;
                self.player.defense = entry.defense;
                events.push(GameEvent::LevelUp {
                    new_level: entry.level,
                });
                break;
            }
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.player.hp <= 0
    }

    pub fn is_game_clear(&self) -> bool {
        self.floor > self.data.map.max_floor
    }

    pub fn max_floor(&self) -> u32 {
        self.data.map.max_floor
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
    use crate::core::master_data::*;
    use crate::map::cell::{MapCell, Terrain};
    use std::collections::HashMap;

    fn make_test_master_data() -> Arc<MasterData> {
        let mut items = HashMap::new();
        items.insert(
            "herb".into(),
            ItemDef {
                name: "回復草".into(),
                symbol: '!',
                category: ItemCategoryDef::Herb,
                effect: ItemEffectDef::Heal(25),
            },
        );
        items.insert(
            "full_heal_herb".into(),
            ItemDef {
                name: "万能草".into(),
                symbol: '!',
                category: ItemCategoryDef::Herb,
                effect: ItemEffectDef::HealFull,
            },
        );
        items.insert(
            "power_herb".into(),
            ItemDef {
                name: "力の草".into(),
                symbol: '!',
                category: ItemCategoryDef::Herb,
                effect: ItemEffectDef::BoostAttack(1),
            },
        );
        items.insert(
            "ration".into(),
            ItemDef {
                name: "携帯食".into(),
                symbol: '%',
                category: ItemCategoryDef::Food,
                effect: ItemEffectDef::Food(500),
            },
        );
        items.insert(
            "big_ration".into(),
            ItemDef {
                name: "保存食".into(),
                symbol: '%',
                category: ItemCategoryDef::Food,
                effect: ItemEffectDef::Food(1000),
            },
        );
        items.insert(
            "light_scroll".into(),
            ItemDef {
                name: "あかりの巻物".into(),
                symbol: '?',
                category: ItemCategoryDef::Scroll,
                effect: ItemEffectDef::RevealMap,
            },
        );
        items.insert(
            "confusion_scroll".into(),
            ItemDef {
                name: "混乱の巻物".into(),
                symbol: '?',
                category: ItemCategoryDef::Scroll,
                effect: ItemEffectDef::ConfuseAll { turns: 10 },
            },
        );
        items.insert(
            "power_scroll".into(),
            ItemDef {
                name: "パワーアップの巻物".into(),
                symbol: '?',
                category: ItemCategoryDef::Scroll,
                effect: ItemEffectDef::TempBoostAttack {
                    amount: 5,
                    turns: 20,
                },
            },
        );
        items.insert(
            "paralysis_staff".into(),
            ItemDef {
                name: "かなしばりの杖".into(),
                symbol: '/',
                category: ItemCategoryDef::Staff,
                effect: ItemEffectDef::Paralyze,
            },
        );
        items.insert(
            "knockback_staff".into(),
            ItemDef {
                name: "ふきとばしの杖".into(),
                symbol: '/',
                category: ItemCategoryDef::Staff,
                effect: ItemEffectDef::Knockback { distance: 5 },
            },
        );
        items.insert(
            "swap_staff".into(),
            ItemDef {
                name: "場所がえの杖".into(),
                symbol: '/',
                category: ItemCategoryDef::Staff,
                effect: ItemEffectDef::SwapPosition,
            },
        );

        let mut equipment = HashMap::new();
        equipment.insert(
            "wooden_sword".into(),
            EquipmentDef {
                name: "木の剣".into(),
                symbol: ')',
                category: EquipCategoryDef::Weapon,
                base_value: 3,
            },
        );
        equipment.insert(
            "iron_sword".into(),
            EquipmentDef {
                name: "鉄の剣".into(),
                symbol: ')',
                category: EquipCategoryDef::Weapon,
                base_value: 6,
            },
        );
        equipment.insert(
            "steel_sword".into(),
            EquipmentDef {
                name: "鋼の剣".into(),
                symbol: ')',
                category: EquipCategoryDef::Weapon,
                base_value: 10,
            },
        );
        equipment.insert(
            "wooden_shield".into(),
            EquipmentDef {
                name: "木の盾".into(),
                symbol: '[',
                category: EquipCategoryDef::Shield,
                base_value: 3,
            },
        );
        equipment.insert(
            "iron_shield".into(),
            EquipmentDef {
                name: "鉄の盾".into(),
                symbol: '[',
                category: EquipCategoryDef::Shield,
                base_value: 5,
            },
        );
        equipment.insert(
            "steel_shield".into(),
            EquipmentDef {
                name: "鋼の盾".into(),
                symbol: '[',
                category: EquipCategoryDef::Shield,
                base_value: 9,
            },
        );

        let mut monsters = HashMap::new();
        monsters.insert(
            "slime".into(),
            MonsterStatsDef {
                name: "スライム".into(),
                symbol: 's',
                hp: 5,
                attack: 2,
                defense: 3,
                exp: 4,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "goblin".into(),
            MonsterStatsDef {
                name: "ゴブリン".into(),
                symbol: 'g',
                hp: 12,
                attack: 5,
                defense: 4,
                exp: 8,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "bat".into(),
            MonsterStatsDef {
                name: "コウモリ".into(),
                symbol: 'b',
                hp: 8,
                attack: 4,
                defense: 5,
                exp: 10,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "golem".into(),
            MonsterStatsDef {
                name: "ゴーレム".into(),
                symbol: 'G',
                hp: 20,
                attack: 8,
                defense: 6,
                exp: 15,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "specter".into(),
            MonsterStatsDef {
                name: "スペクター".into(),
                symbol: 'S',
                hp: 22,
                attack: 11,
                defense: 8,
                exp: 18,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "drake".into(),
            MonsterStatsDef {
                name: "ドレイク".into(),
                symbol: 'D',
                hp: 45,
                attack: 20,
                defense: 15,
                exp: 50,
                ai_type: AiTypeDef::Standard,
            },
        );
        monsters.insert(
            "imp".into(),
            MonsterStatsDef {
                name: "インプ".into(),
                symbol: 'i',
                hp: 18,
                attack: 10,
                defense: 7,
                exp: 20,
                ai_type: AiTypeDef::Ranged,
            },
        );
        monsters.insert(
            "guardian".into(),
            MonsterStatsDef {
                name: "ガーディアン".into(),
                symbol: 'W',
                hp: 60,
                attack: 25,
                defense: 30,
                exp: 100,
                ai_type: AiTypeDef::Standard,
            },
        );

        let floors = FloorData {
            monster_table: vec![
                MonsterTableEntry {
                    floors: FloorRange(1, 2),
                    monsters: vec!["slime".into(), "goblin".into()],
                },
                MonsterTableEntry {
                    floors: FloorRange(3, 3),
                    monsters: vec!["slime".into(), "goblin".into(), "bat".into()],
                },
                MonsterTableEntry {
                    floors: FloorRange(4, 5),
                    monsters: vec!["goblin".into(), "bat".into(), "golem".into()],
                },
                MonsterTableEntry {
                    floors: FloorRange(6, 7),
                    monsters: vec!["specter".into(), "golem".into(), "imp".into()],
                },
                MonsterTableEntry {
                    floors: FloorRange(8, 9),
                    monsters: vec!["specter".into(), "drake".into(), "imp".into()],
                },
                MonsterTableEntry {
                    floors: FloorRange(10, 999),
                    monsters: vec!["drake".into(), "imp".into(), "guardian".into()],
                },
            ],
            monster_counts: vec![
                MonsterCountEntry {
                    floors: FloorRange(1, 3),
                    count: 8,
                },
                MonsterCountEntry {
                    floors: FloorRange(4, 6),
                    count: 10,
                },
                MonsterCountEntry {
                    floors: FloorRange(7, 9),
                    count: 12,
                },
                MonsterCountEntry {
                    floors: FloorRange(10, 999),
                    count: 15,
                },
            ],
            herb_spawns: vec![HerbSpawnEntry {
                floors: FloorRange(1, 999),
                count: 3,
            }],
            food_spawns: vec![
                FoodSpawnEntry {
                    floors: FloorRange(1, 3),
                    count: 2,
                    items: vec![
                        FoodSpawnItem {
                            id: "big_ration".into(),
                            weight: 0.3,
                        },
                        FoodSpawnItem {
                            id: "ration".into(),
                            weight: 0.7,
                        },
                    ],
                },
                FoodSpawnEntry {
                    floors: FloorRange(4, 999),
                    count: 1,
                    items: vec![
                        FoodSpawnItem {
                            id: "big_ration".into(),
                            weight: 0.3,
                        },
                        FoodSpawnItem {
                            id: "ration".into(),
                            weight: 0.7,
                        },
                    ],
                },
            ],
            scroll_spawns: ScrollSpawnDef {
                min: 1,
                max: 2,
                pool: vec![
                    "light_scroll".into(),
                    "confusion_scroll".into(),
                    "power_scroll".into(),
                ],
            },
            staff_spawns: StaffSpawnDef {
                chance: 0.5,
                min_charges: 3,
                max_charges: 6,
                pool: vec![
                    "paralysis_staff".into(),
                    "knockback_staff".into(),
                    "swap_staff".into(),
                ],
            },
            equipment_spawns: vec![
                EquipSpawnEntry {
                    floors: FloorRange(1, 3),
                    chance: 0.5,
                    weapons: vec!["wooden_sword".into()],
                    shields: vec!["wooden_shield".into()],
                    max_enhancement: 3,
                },
                EquipSpawnEntry {
                    floors: FloorRange(4, 6),
                    chance: 0.5,
                    weapons: vec!["iron_sword".into()],
                    shields: vec!["iron_shield".into()],
                    max_enhancement: 3,
                },
                EquipSpawnEntry {
                    floors: FloorRange(7, 999),
                    chance: 0.5,
                    weapons: vec!["steel_sword".into()],
                    shields: vec!["steel_shield".into()],
                    max_enhancement: 3,
                },
            ],
        };

        let player = PlayerData {
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
            level_table: vec![
                LevelEntry {
                    level: 1,
                    req_exp: 0,
                    max_hp: 30,
                    attack: 8,
                    defense: 5,
                },
                LevelEntry {
                    level: 2,
                    req_exp: 30,
                    max_hp: 35,
                    attack: 10,
                    defense: 6,
                },
                LevelEntry {
                    level: 3,
                    req_exp: 70,
                    max_hp: 40,
                    attack: 12,
                    defense: 7,
                },
                LevelEntry {
                    level: 4,
                    req_exp: 120,
                    max_hp: 45,
                    attack: 14,
                    defense: 8,
                },
                LevelEntry {
                    level: 5,
                    req_exp: 200,
                    max_hp: 50,
                    attack: 16,
                    defense: 9,
                },
            ],
        };

        let balance = BalanceData {
            min_damage: 1,
            thrown_non_weapon_damage: 2,
            fullness_decrease_per_turn: 1,
            starvation_damage: 1,
            detection_range: 5,
            staff_range: 20,
            throw_range: 10,
        };

        let map = MapData {
            width: 100,
            height: 50,
            min_room_size: 3,
            min_aisle_size: 2,
            cut_trial: 9,
            max_random_aisles: 6,
            max_floor: 10,
        };

        Arc::new(MasterData {
            items,
            equipment,
            monsters,
            floors,
            player,
            balance,
            map,
        })
    }

    fn make_test_item(data: &MasterData, id: &str) -> Item {
        let def = data.items.get(id).unwrap();
        Item::from_def(id, def, None)
    }

    fn make_test_item_with_charges(data: &MasterData, id: &str, charges: i32) -> Item {
        let def = data.items.get(id).unwrap();
        Item::from_def(id, def, Some(charges))
    }

    fn make_test_equip(data: &MasterData, id: &str, enhancement: i32) -> Equipment {
        let def = data.equipment.get(id).unwrap();
        Equipment::from_def(id, def, enhancement)
    }

    fn make_test_monster(data: &MasterData, id: &str, pos: Position) -> Monster {
        let def = data.monsters.get(id).unwrap();
        Monster::from_stats_def(def, pos)
    }

    fn make_test_state() -> GameState {
        let data = make_test_master_data();
        let mut map = GameMap::new(10, 10, Position::new(8, 8));
        for y in 1..=8 {
            for x in 1..=8 {
                map.set(
                    x,
                    y,
                    MapCell {
                        terrain: Terrain::Floor { room_id: 1 },
                    },
                );
            }
        }
        map.set(
            8,
            8,
            MapCell {
                terrain: Terrain::Exit,
            },
        );

        let mut player = Player::new(&data.player);
        player.pos = Position::new(2, 2);

        let mut state = GameState {
            player,
            monsters: Vec::new(),
            floor_items: Vec::new(),
            floor_equips: Vec::new(),
            map,
            visibility: Visibility::new(),
            floor: 1,
            turn: 0,
            data,
        };
        state.visibility.update(&state.player.pos, &state.map);
        state
    }

    // --- Movement ---

    #[test]
    fn move_to_walkable_tile() {
        let mut state = make_test_state();
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.player.pos, Position::new(3, 2));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerMoved { .. })));
    }

    #[test]
    fn move_into_wall_stays() {
        let mut state = make_test_state();
        state.player.pos = Position::new(1, 1);
        state
            .process_turn(GameCommand::Move(Direction::Left))
            .unwrap();
        assert_eq!(state.player.pos, Position::new(1, 1));
    }

    #[test]
    fn move_onto_exit_advances_floor() {
        let mut state = make_test_state();
        state.player.pos = Position::new(7, 8);
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.floor, 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::FloorAdvance { .. })));
    }

    #[test]
    fn move_onto_exit_floor_10_game_clear() {
        let mut state = make_test_state();
        state.floor = 10;
        state.player.pos = Position::new(7, 8);
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::GameClear)));
    }

    // --- Combat ---

    #[test]
    fn attack_monster_deals_damage() {
        let mut state = make_test_state();
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));

        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerAttacked { damage: 5, .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::MonsterDefeated { .. })));
        assert!(state.monsters.is_empty());
    }

    #[test]
    fn attack_monster_gains_exp() {
        let mut state = make_test_state();
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));
        state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.player.exp, 4);
    }

    #[test]
    fn attack_releases_paralysis() {
        let mut state = make_test_state();
        let mut monster = make_test_monster(&state.data, "golem", Position::new(3, 2));
        monster.status.paralyzed = true;
        state.monsters.push(monster);

        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(!state.monsters[0].status.paralyzed);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::Message(m) if m.contains("金縛り"))));
    }

    // --- Item pickup ---

    #[test]
    fn item_pickup_on_move() {
        let mut state = make_test_state();
        state.floor_items.push(FloorItem {
            item: make_test_item(&state.data, "herb"),
            pos: Position::new(3, 2),
        });
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemPickedUp { .. })));
        assert_eq!(state.player.inventory.len(), 1);
        assert!(state.floor_items.is_empty());
    }

    #[test]
    fn item_pickup_inventory_full() {
        let mut state = make_test_state();
        let max_inv = state.data.player.max_inventory;
        for _ in 0..max_inv {
            state
                .player
                .inventory
                .push(make_test_item(&state.data, "herb"));
        }
        state.floor_items.push(FloorItem {
            item: make_test_item(&state.data, "herb"),
            pos: Position::new(3, 2),
        });
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::InventoryFull)));
        assert_eq!(state.floor_items.len(), 1);
    }

    // --- Item usage ---

    #[test]
    fn use_herb_heals() {
        let mut state = make_test_state();
        state.player.hp = 10;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "herb"));
        let events = state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.hp, 30);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemUsed { .. })));
        assert!(state.player.inventory.is_empty());
    }

    #[test]
    fn use_herb_at_max_hp_boosts_max() {
        let mut state = make_test_state();
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "herb"));
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.max_hp, 31);
        assert_eq!(state.player.hp, 31);
    }

    #[test]
    fn use_food_restores_fullness() {
        let mut state = make_test_state();
        state.player.fullness = 500;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "ration"));
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.fullness, 999);
    }

    #[test]
    fn use_power_herb_boosts_attack() {
        let mut state = make_test_state();
        let old_attack = state.player.attack;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "power_herb"));
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
        state.player.exp = 29;
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));
        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.player.level, 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::LevelUp { new_level: 2 })));
    }

    // --- Equipment ---

    #[test]
    fn equip_weapon_from_inventory() {
        let mut state = make_test_state();
        let sword = make_test_equip(&state.data, "iron_sword", 1);
        let item = equipment_to_item(&sword);
        state.player.inventory.push(item);

        let events = state.process_turn(GameCommand::EquipWeapon(0)).unwrap();
        assert!(state.player.weapon.is_some());
        assert_eq!(state.player.weapon.as_ref().unwrap().base_value, 6);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::Equipped { .. })));
        assert!(state.player.inventory.is_empty());
    }

    #[test]
    fn equip_weapon_swaps_old_to_inventory() {
        let mut state = make_test_state();
        state.player.weapon = Some(make_test_equip(&state.data, "wooden_sword", 0));
        let new_sword = equipment_to_item(&make_test_equip(&state.data, "iron_sword", 0));
        state.player.inventory.push(new_sword);

        state.process_turn(GameCommand::EquipWeapon(0)).unwrap();
        assert_eq!(state.player.weapon.as_ref().unwrap().base_value, 6);
        assert_eq!(state.player.inventory.len(), 1);
    }

    // --- Staff usage ---

    #[test]
    fn use_paralysis_staff_paralyzes_monster() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(make_test_item_with_charges(
            &state.data,
            "paralysis_staff",
            3,
        ));
        state
            .monsters
            .push(make_test_monster(&state.data, "golem", Position::new(5, 2)));

        state.process_turn(GameCommand::UseStaff(0)).unwrap();
        assert!(state.monsters[0].status.paralyzed);
    }

    #[test]
    fn use_staff_decreases_charges() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(make_test_item_with_charges(
            &state.data,
            "paralysis_staff",
            3,
        ));
        state
            .monsters
            .push(make_test_monster(&state.data, "golem", Position::new(5, 2)));

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
        state
            .player
            .inventory
            .push(make_test_item_with_charges(&state.data, "swap_staff", 3));
        let mut monster = make_test_monster(&state.data, "golem", Position::new(5, 2));
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
        let mut monster = make_test_monster(&state.data, "slime", Position::new(3, 2));
        monster.status.paralyzed = true;
        let pos_before = monster.pos;
        state.monsters.push(monster);

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
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
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::RequestInventory)));
        assert_eq!(state.turn, 0);
    }

    // --- Monster count per floor ---

    #[test]
    fn monster_count_for_floor_values() {
        let state = make_test_state();
        assert_eq!(state.monster_count_for_floor(1), 8);
        assert_eq!(state.monster_count_for_floor(3), 8);
        assert_eq!(state.monster_count_for_floor(4), 10);
        assert_eq!(state.monster_count_for_floor(7), 12);
        assert_eq!(state.monster_count_for_floor(10), 15);
    }

    // --- Confused monster ---

    #[test]
    fn confused_monster_does_not_attack() {
        let mut state = make_test_state();
        let mut monster = make_test_monster(&state.data, "slime", Position::new(3, 2));
        monster.status.confused = Some(5);
        state.monsters.push(monster);

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.player.hp, hp_before);
    }

    // === 境界条件テスト ===

    #[test]
    fn attack_damage_clamped_to_one_when_defense_exceeds_attack() {
        let mut state = make_test_state();
        state.player.attack = 1;
        state.player.weapon = None;
        state.monsters.push(make_test_monster(
            &state.data,
            "guardian",
            Position::new(3, 2),
        ));

        let events = state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerAttacked { damage: 1, .. })));
        assert_eq!(state.monsters[0].hp, 60 - 1);
    }

    #[test]
    fn monster_damage_clamped_to_one_when_defense_exceeds_attack() {
        let mut state = make_test_state();
        state.player.shield = Some(make_test_equip(&state.data, "steel_shield", 0));
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));

        let hp_before = state.player.hp;
        state.process_turn(GameCommand::Wait).unwrap();
        assert_eq!(state.player.hp, hp_before - 1);
        assert!(state.player.hp > 0);
    }

    #[test]
    fn use_herb_clamps_to_max_hp() {
        let mut state = make_test_state();
        state.player.hp = 29;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "herb"));
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.hp, 30);
        assert_eq!(state.player.max_hp, 30);
    }

    #[test]
    fn use_food_at_max_fullness() {
        let mut state = make_test_state();
        state.player.fullness = 1000;
        state.player.max_fullness = 1000;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "ration"));
        state.process_turn(GameCommand::UseItem(0)).unwrap();
        assert_eq!(state.player.fullness, 999);
    }

    #[test]
    fn use_staff_with_zero_charges() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(make_test_item_with_charges(
            &state.data,
            "paralysis_staff",
            0,
        ));
        state
            .monsters
            .push(make_test_monster(&state.data, "golem", Position::new(5, 2)));

        let events = state.process_turn(GameCommand::UseStaff(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::Message(m) if m.contains("何も起こらなかった"))));
        assert!(!state.monsters[0].status.paralyzed);
    }

    #[test]
    fn use_item_out_of_bounds_index() {
        let mut state = make_test_state();
        let result = state.process_turn(GameCommand::UseItem(99));
        assert!(matches!(result, Err(TurnError::InvalidInventoryIndex(99))));
        assert_eq!(state.turn, 0);
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

    #[test]
    fn level_up_at_exact_threshold() {
        let mut state = make_test_state();
        state.player.exp = 30;
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));
        state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.player.level, 2);
    }

    #[test]
    fn no_level_up_below_threshold() {
        let mut state = make_test_state();
        state.player.exp = 25;
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(3, 2)));
        state
            .process_turn(GameCommand::Move(Direction::Right))
            .unwrap();
        assert_eq!(state.player.level, 1);
    }

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

    // === 投擲テスト ===

    #[test]
    fn throw_herb_hits_monster_deals_damage_and_heals() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "herb"));
        let mut monster = make_test_monster(&state.data, "slime", Position::new(5, 2));
        monster.hp = 4;
        state.monsters.push(monster);

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemThrown { .. })));
        assert_eq!(state.monsters[0].hp, 5);
        assert!(state.player.inventory.is_empty());
    }

    #[test]
    fn throw_toward_wall_item_lost() {
        let mut state = make_test_state();
        state.player.pos = Position::new(1, 2);
        state.player.direction = Direction::Left;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "herb"));

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::ItemThrown { ref result_desc, .. } if result_desc.contains("壁"))));
        assert!(state.player.inventory.is_empty());
        assert!(state.floor_items.is_empty());
    }

    #[test]
    fn throw_misses_drops_on_ground() {
        let data = make_test_master_data();
        let mut map = GameMap::new(20, 10, Position::new(18, 8));
        for y in 1..=8 {
            for x in 1..=18 {
                map.set(
                    x,
                    y,
                    MapCell {
                        terrain: Terrain::Floor { room_id: 1 },
                    },
                );
            }
        }
        let mut player = Player::new(&data.player);
        player.pos = Position::new(2, 2);
        player.direction = Direction::Right;
        player.inventory.push(make_test_item(&data, "herb"));

        let mut state = GameState {
            player,
            monsters: Vec::new(),
            floor_items: Vec::new(),
            floor_equips: Vec::new(),
            map,
            visibility: Visibility::new(),
            floor: 1,
            turn: 0,
            data,
        };
        state.visibility.update(&state.player.pos, &state.map);

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::ItemThrown { ref result_desc, .. } if result_desc.contains("地面"))));
        assert!(state.player.inventory.is_empty());
        assert_eq!(state.floor_items.len(), 1);
        assert_eq!(state.floor_items[0].pos, Position::new(12, 2));
    }

    #[test]
    fn throw_weapon_deals_half_attack() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        let weapon_item = Item {
            id: "iron_sword".into(),
            name: "鉄の剣+2".into(),
            symbol: ')',
            category: ItemCategory::Weapon,
            effect: ItemEffect::Heal(0),
            equip_data: Some(EquipData {
                base_value: 6,
                enhancement: 2,
            }),
        };
        state.player.inventory.push(weapon_item);
        state.monsters.push(make_test_monster(
            &state.data,
            "specter",
            Position::new(5, 2),
        ));

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemThrown { .. })));
        assert_eq!(state.monsters[0].hp, 22 - 4);
    }

    #[test]
    fn throw_power_herb_boosts_monster_attack() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "power_herb"));
        state
            .monsters
            .push(make_test_monster(&state.data, "slime", Position::new(4, 2)));

        let original_attack = state.monsters[0].attack;
        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemThrown { .. })));
        assert_eq!(state.monsters[0].attack, original_attack + 1);
    }

    #[test]
    fn throw_kills_monster_grants_exp() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state
            .player
            .inventory
            .push(make_test_item(&state.data, "ration"));
        let mut monster = make_test_monster(&state.data, "slime", Position::new(4, 2));
        monster.hp = 2;
        let exp = monster.exp;
        state.monsters.push(monster);

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::MonsterDefeated { .. })));
        assert_eq!(state.player.exp, exp);
        assert!(state.monsters.is_empty());
    }

    #[test]
    fn throw_invalid_index_returns_error() {
        let mut state = make_test_state();
        let result = state.process_turn(GameCommand::ThrowItem(0));
        assert!(result.is_err());
    }

    #[test]
    fn throw_staff_deals_flat_damage() {
        let mut state = make_test_state();
        state.player.direction = Direction::Right;
        state.player.inventory.push(make_test_item_with_charges(
            &state.data,
            "paralysis_staff",
            3,
        ));
        state.monsters.push(make_test_monster(
            &state.data,
            "specter",
            Position::new(5, 2),
        ));

        let events = state.process_turn(GameCommand::ThrowItem(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::ItemThrown { .. })));
        assert_eq!(state.monsters[0].hp, 22 - 2);
        assert!(!state.monsters[0].status.paralyzed);
    }
}
