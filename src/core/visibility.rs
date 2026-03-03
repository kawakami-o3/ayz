use std::collections::HashSet;

use super::types::Position;
use crate::map::cell::GameMap;

pub struct Visibility {
    pub visible: HashSet<Position>,
    pub visited: HashSet<Position>,
    pub full_map: bool,
}

impl Visibility {
    pub fn new() -> Self {
        Visibility {
            visible: HashSet::new(),
            visited: HashSet::new(),
            full_map: false,
        }
    }

    pub fn update(&mut self, player_pos: &Position, map: &GameMap) {
        self.visible.clear();

        if self.full_map {
            // Reveal everything (e.g. scroll of light)
            for y in 0..map.height {
                for x in 0..map.width {
                    let pos = Position::new(x as i32, y as i32);
                    if map.is_walkable(&pos) || map.is_exit(&pos) {
                        self.visible.insert(pos);
                    }
                }
            }
        } else if map.get(player_pos).map_or(false, |c| c.is_room()) {
            // In a room: reveal entire room
            self.reveal_room(player_pos, map);
        } else {
            // In corridor: reveal adjacent 1 tile
            self.reveal_adjacent(player_pos);
        }

        // Add visible to visited
        for pos in &self.visible {
            self.visited.insert(*pos);
        }
    }

    fn reveal_room(&mut self, player_pos: &Position, map: &GameMap) {
        // Add adjacent tiles first (to see corridor entrances)
        self.reveal_adjacent(player_pos);

        // Add all tiles in the same room
        for y in 0..map.height {
            for x in 0..map.width {
                let pos = Position::new(x as i32, y as i32);
                if map.is_same_room(player_pos, &pos) {
                    self.visible.insert(pos);
                }
            }
        }

        // Also reveal exit if in same room
        if map.is_same_room(player_pos, &map.exit_pos) {
            self.visible.insert(map.exit_pos);
        }
        if map.is_exit(&map.exit_pos) {
            // Check adjacency to any visible tile
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let adj = map.exit_pos.plus(&Position::new(dx, dy));
                    if self.visible.contains(&adj) {
                        self.visible.insert(map.exit_pos);
                    }
                }
            }
        }
    }

    fn reveal_adjacent(&mut self, center: &Position) {
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                self.visible.insert(center.plus(&Position::new(dx, dy)));
            }
        }
    }

    pub fn is_visible(&self, pos: &Position) -> bool {
        self.visible.contains(pos)
    }

    pub fn is_visited(&self, pos: &Position) -> bool {
        self.visited.contains(pos)
    }

    pub fn reset_for_new_floor(&mut self) {
        self.visible.clear();
        self.visited.clear();
        self.full_map = false;
    }
}
