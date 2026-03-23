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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::cell::{GameMap, MapCell, Terrain};

    /// Create a small map with:
    /// - Room 1 at (1,1)-(3,3)
    /// - Aisle at (4,2)
    /// - Room 2 at (5,1)-(7,3)
    fn make_two_room_map() -> GameMap {
        let mut map = GameMap::new(9, 5, Position::new(6, 2));
        // Room 1
        for y in 1..=3 {
            for x in 1..=3 {
                map.set(x, y, MapCell { terrain: Terrain::Floor { room_id: 1 } });
            }
        }
        // Aisle connecting rooms
        map.set(4, 2, MapCell { terrain: Terrain::Aisle });
        // Room 2
        for y in 1..=3 {
            for x in 5..=7 {
                map.set(x, y, MapCell { terrain: Terrain::Floor { room_id: 2 } });
            }
        }
        // Exit
        map.set(6, 2, MapCell { terrain: Terrain::Exit });
        map
    }

    #[test]
    fn new_visibility_is_empty() {
        let vis = Visibility::new();
        assert!(vis.visible.is_empty());
        assert!(vis.visited.is_empty());
        assert!(!vis.full_map);
    }

    #[test]
    fn update_in_room_reveals_entire_room() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();
        let player_pos = Position::new(2, 2); // center of room 1

        vis.update(&player_pos, &map);

        // All tiles of room 1 should be visible
        for y in 1..=3 {
            for x in 1..=3 {
                assert!(vis.is_visible(&Position::new(x, y)),
                    "Room 1 tile ({},{}) should be visible", x, y);
            }
        }
        // Room 2 tiles should NOT be visible
        assert!(!vis.is_visible(&Position::new(6, 2)));
    }

    #[test]
    fn update_in_room_reveals_adjacent_aisle() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();
        let player_pos = Position::new(3, 2); // edge of room 1, next to aisle

        vis.update(&player_pos, &map);

        // Aisle at (4,2) is adjacent to room 1 tile (3,2)
        assert!(vis.is_visible(&Position::new(4, 2)),
            "Aisle adjacent to room should be visible");
    }

    #[test]
    fn update_in_corridor_reveals_3x3() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();
        let player_pos = Position::new(4, 2); // in aisle

        vis.update(&player_pos, &map);

        // 3x3 area around (4,2) should be visible
        for dy in -1..=1 {
            for dx in -1..=1 {
                let pos = Position::new(4 + dx, 2 + dy);
                assert!(vis.is_visible(&pos),
                    "({},{}) should be visible from corridor", pos.x, pos.y);
            }
        }
    }

    #[test]
    fn full_map_reveals_all_walkable() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();
        vis.full_map = true;

        vis.update(&Position::new(2, 2), &map);

        // All room tiles and aisle should be visible
        assert!(vis.is_visible(&Position::new(1, 1))); // room 1
        assert!(vis.is_visible(&Position::new(4, 2))); // aisle
        assert!(vis.is_visible(&Position::new(5, 1))); // room 2
        // Walls should NOT be visible
        assert!(!vis.is_visible(&Position::new(0, 0)));
    }

    #[test]
    fn visited_persists_across_updates() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();

        // First update: in room 1
        vis.update(&Position::new(2, 2), &map);
        assert!(vis.is_visited(&Position::new(1, 1)));

        // Second update: in aisle
        vis.update(&Position::new(4, 2), &map);
        // Room 1 tiles no longer visible but still visited
        assert!(!vis.is_visible(&Position::new(1, 1)));
        assert!(vis.is_visited(&Position::new(1, 1)));
    }

    #[test]
    fn reset_for_new_floor_clears_all() {
        let map = make_two_room_map();
        let mut vis = Visibility::new();
        vis.full_map = true;
        vis.update(&Position::new(2, 2), &map);

        vis.reset_for_new_floor();

        assert!(vis.visible.is_empty());
        assert!(vis.visited.is_empty());
        assert!(!vis.full_map);
    }
}
