use rand::prelude::*;
use std::collections::HashSet;

use crate::core::types::Position;
use super::cell::GameMap;

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
