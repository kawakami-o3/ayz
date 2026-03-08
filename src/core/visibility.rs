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
        // Collect all tiles in the same room
        let mut room_tiles = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                let pos = Position::new(x as i32, y as i32);
                if map.is_same_room(player_pos, &pos) {
                    self.visible.insert(pos);
                    room_tiles.push(pos);
                }
            }
        }

        // Reveal corridor/aisle tiles adjacent to the room (entrance tiles)
        for room_pos in &room_tiles {
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let adj = room_pos.plus(&Position::new(dx, dy));
                    if let Some(cell) = map.get(&adj) {
                        if matches!(cell.terrain, crate::map::cell::Terrain::Aisle | crate::map::cell::Terrain::Exit) {
                            self.visible.insert(adj);
                        }
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
