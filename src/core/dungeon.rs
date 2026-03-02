use std::collections::HashSet;

use rand::prelude::*;

use super::entity::{Monster, Player};
use super::turn::{GameCommand, GameEvent};
use super::types::{Direction, Position};
use crate::map::cell::GameMap;
use crate::map::generator;
use crate::map::spawn::calc_spawn_pos;

pub struct GameState {
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub map: GameMap,
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
            map,
            floor: 1,
            max_floor: 10,
            turn: 0,
        };
        state.spawn_entities();
        state
    }

    fn spawn_entities(&mut self) {
        let monster_count = 10;
        let mut occupied = HashSet::new();

        // Spawn monsters
        self.monsters.clear();
        for _ in 0..monster_count {
            let pos = calc_spawn_pos(&self.map, &occupied);
            occupied.insert(pos);
            self.monsters.push(Monster::new(pos));
        }

        // Spawn player
        let player_pos = calc_spawn_pos(&self.map, &occupied);
        self.player.pos = player_pos;
    }

    pub fn process_turn(&mut self, command: GameCommand) -> Vec<GameEvent> {
        let mut events = Vec::new();

        match command {
            GameCommand::Move(dir) => {
                self.player.direction = dir;
                let target = self.player.pos.plus(&dir.to_offset());

                if let Some(monster_idx) = self.monster_at(&target) {
                    // Attack
                    let damage = std::cmp::max(1, self.player.attack - self.monsters[monster_idx].defense);
                    self.monsters[monster_idx].hp -= damage;
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
                        self.check_level_up(&mut events);
                    }
                } else if self.map.is_exit(&target) {
                    self.floor += 1;
                    if self.floor > self.max_floor {
                        events.push(GameEvent::GameClear);
                        return events;
                    } else {
                        events.push(GameEvent::FloorAdvance { new_floor: self.floor });
                        self.map = generator::generate();
                        self.spawn_entities();
                        return events;
                    }
                } else if self.map.is_walkable(&target) && self.monster_at(&target).is_none() {
                    let from = self.player.pos;
                    self.player.pos = target;
                    events.push(GameEvent::PlayerMoved { from, to: target });
                }
            }
            GameCommand::Wait => {}
            GameCommand::Quit => return events,
        }

        // Check game over after player action
        if self.player.hp <= 0 {
            events.push(GameEvent::GameOver);
            return events;
        }

        // Monster turns
        self.process_monster_turns(&mut events);

        // Check game over after monster actions
        if self.player.hp <= 0 {
            events.push(GameEvent::GameOver);
        }

        self.turn += 1;
        events
    }

    fn process_monster_turns(&mut self, events: &mut Vec<GameEvent>) {
        let mut rng = thread_rng();

        for i in 0..self.monsters.len() {
            let monster_pos = self.monsters[i].pos;

            // Check if adjacent to player -> attack
            if monster_pos.manhattan_distance(&self.player.pos) == 1
                || (monster_pos.x - self.player.pos.x).abs() <= 1
                    && (monster_pos.y - self.player.pos.y).abs() <= 1
            {
                let damage = std::cmp::max(1, self.monsters[i].attack - self.player.defense);
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
            if target == ppos {
                // Adjacent to player, don't move (will attack next turn)
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
            // (required_exp, max_hp, attack, defense, level)
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
