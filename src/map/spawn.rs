use rand::prelude::*;
use std::collections::HashSet;

use super::cell::GameMap;
use crate::core::types::Position;

pub fn calc_spawn_pos(map: &GameMap, occupied: &HashSet<Position>) -> Position {
    let candidates: Vec<Position> = map
        .room_positions()
        .into_iter()
        .filter(|p| !occupied.contains(p) && !map.is_exit(p))
        .collect();

    if candidates.is_empty() {
        return Position::zero();
    }

    let mut rng = thread_rng();
    candidates[rng.gen_range(0..candidates.len())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::cell::{MapCell, Terrain};

    fn make_small_map() -> GameMap {
        // 5x5 map with a 3x3 room (room_id=1) at (1,1)-(3,3) and exit at (4,4)
        let mut map = GameMap::new(5, 5, Position::new(4, 4));
        for y in 1..=3 {
            for x in 1..=3 {
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
            4,
            4,
            MapCell {
                terrain: Terrain::Exit,
            },
        );
        map
    }

    #[test]
    fn spawn_pos_avoids_occupied_and_exit() {
        let map = make_small_map();
        let mut occupied = HashSet::new();
        // Occupy all room positions except (1,1)
        for y in 1..=3 {
            for x in 1..=3 {
                if !(x == 1 && y == 1) {
                    occupied.insert(Position::new(x as i32, y as i32));
                }
            }
        }
        let pos = calc_spawn_pos(&map, &occupied);
        assert_eq!(pos, Position::new(1, 1));
    }

    #[test]
    fn spawn_pos_returns_zero_when_no_candidates() {
        let map = GameMap::new(3, 3, Position::new(0, 0)); // all walls
        let occupied = HashSet::new();
        let pos = calc_spawn_pos(&map, &occupied);
        assert_eq!(pos, Position::zero());
    }

    #[test]
    fn spawn_pos_is_in_room() {
        let map = make_small_map();
        let occupied = HashSet::new();
        let pos = calc_spawn_pos(&map, &occupied);
        // Must be a room position (not wall, not exit)
        assert!(pos.x >= 1 && pos.x <= 3 && pos.y >= 1 && pos.y <= 3);
    }
}
