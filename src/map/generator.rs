use rand::prelude::*;

use crate::core::types::Position;
use super::cell::{GameMap, MapCell, Terrain};

const MIN_ROOM_SIZE: usize = 3;
const MIN_AISLE_SIZE: usize = 2;
const MIN_CUT_SIZE: usize = 2 * (MIN_ROOM_SIZE + MIN_AISLE_SIZE * 2);
const CUT_TRIAL: usize = 9;

#[derive(Clone, Debug)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    idx: usize,
}

impl Room {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            idx: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct Link {
    up: Vec<usize>,
    down: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
}

impl Link {
    fn new() -> Link {
        Link {
            up: Vec::new(),
            down: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct Area {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    idx: usize,
    link: Link,
    room: Room,
}

impl Area {
    fn new() -> Area {
        Area {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            idx: 0,
            link: Link::new(),
            room: Room::new(),
        }
    }

    fn is_link(&self, target: &Area, cut_type: &CutType) -> bool {
        match cut_type {
            CutType::Horizontal => {
                !(self.y > (target.y + target.h) || target.y > (self.y + self.h))
            }
            CutType::Vertical => {
                !(self.x > (target.x + target.w) || target.x > (self.x + self.w))
            }
        }
    }
}

fn calc_weight(a: &Area) -> usize {
    a.w * a.h
}

fn choose(areas: &[Area]) -> usize {
    let mut rnd = thread_rng();
    let mut total_weight = 0;
    for a in areas {
        if a.w >= MIN_CUT_SIZE || a.h >= MIN_CUT_SIZE {
            total_weight += calc_weight(a);
        }
    }
    let target = rnd.gen_range(0..total_weight);

    let mut sum = 0;
    for (i, a) in areas.iter().enumerate() {
        if a.w >= MIN_CUT_SIZE || a.h >= MIN_CUT_SIZE {
            sum += calc_weight(a);
            if target < sum {
                return i;
            }
        }
    }
    0
}

#[derive(PartialEq, Debug)]
enum CutType {
    Vertical,
    Horizontal,
}

fn calc_cut_size(size: usize) -> usize {
    size / 2
}

fn cut_areas(areas: &mut Vec<Area>) {
    let mut rnd = thread_rng();

    for _i in 0..CUT_TRIAL {
        let idx = choose(areas);

        if areas[idx].w < MIN_CUT_SIZE && areas[idx].h < MIN_CUT_SIZE {
            continue;
        }

        let mut base = areas[idx].clone();

        let mut cut_type_list = Vec::new();
        if base.w >= MIN_CUT_SIZE {
            cut_type_list.push(CutType::Vertical);
        }
        if base.h >= MIN_CUT_SIZE {
            cut_type_list.push(CutType::Horizontal);
        }
        let cut_type = &cut_type_list[rnd.gen_range(0..cut_type_list.len())];

        let new_idx = areas.len();

        let mut area = Area::new();
        area.idx = new_idx;

        match cut_type {
            CutType::Horizontal => {
                let new_size = calc_cut_size(base.h);
                area.x = base.x;
                area.y = base.y + new_size;
                area.w = base.w;
                area.h = base.h - new_size;
                base.h = new_size;
            }
            CutType::Vertical => {
                let new_size = calc_cut_size(base.w);
                area.x = base.x + new_size;
                area.y = base.y;
                area.w = base.w - new_size;
                area.h = base.h;
                base.w = new_size;
            }
        }

        match cut_type {
            CutType::Horizontal => {
                for i in base.link.down.clone() {
                    let mut target = None;
                    let mut link = areas[i].link.clone();
                    for (ii, j) in link.up.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        area.link.down.push(areas[i].idx);
                        link.up.remove(target_idx);
                        link.up.push(new_idx);
                        areas[i].link = link;
                    }
                }

                base.link.down = vec![new_idx];
                area.link.up = vec![idx];

                {
                    let old_link = base.link.right.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], &CutType::Horizontal) {
                            if let Some(ii) = base.link.right.iter().position(|x| *x == i) {
                                base.link.right.remove(ii);
                            }
                            if let Some(ii) = areas[i].link.left.iter().position(|x| *x == idx) {
                                areas[i].link.left.remove(ii);
                            }
                        }
                        if area.is_link(&areas[i], &CutType::Horizontal) {
                            area.link.right.push(i);
                            areas[i].link.left.push(new_idx);
                        }
                    }
                }

                {
                    let old_link = base.link.left.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], &CutType::Horizontal) {
                            if let Some(ii) = base.link.left.iter().position(|x| *x == i) {
                                base.link.left.remove(ii);
                            }
                            if let Some(ii) = areas[i].link.right.iter().position(|x| *x == idx) {
                                areas[i].link.right.remove(ii);
                            }
                        }
                        if area.is_link(&areas[i], &CutType::Horizontal) {
                            area.link.left.push(i);
                            areas[i].link.right.push(new_idx);
                        }
                    }
                }
            }
            CutType::Vertical => {
                for i in base.link.right.clone() {
                    let mut target = None;
                    let mut link = areas[i].link.clone();
                    for (ii, j) in link.left.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        area.link.right.push(areas[i].idx);
                        link.left.remove(target_idx);
                        link.left.push(new_idx);
                        areas[i].link = link;
                    }
                }

                base.link.right = vec![new_idx];
                area.link.left = vec![idx];

                {
                    let old_link = base.link.up.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], &CutType::Vertical) {
                            if let Some(ii) = base.link.up.iter().position(|x| *x == i) {
                                base.link.up.remove(ii);
                            }
                            if let Some(ii) = areas[i].link.down.iter().position(|x| *x == idx) {
                                areas[i].link.down.remove(ii);
                            }
                        }
                        if area.is_link(&areas[i], &CutType::Vertical) {
                            area.link.up.push(i);
                            areas[i].link.down.push(new_idx);
                        }
                    }
                }

                {
                    let old_link = base.link.down.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], &CutType::Vertical) {
                            if let Some(ii) = base.link.down.iter().position(|x| *x == i) {
                                base.link.down.remove(ii);
                            }
                            if let Some(ii) = areas[i].link.up.iter().position(|x| *x == idx) {
                                areas[i].link.up.remove(ii);
                            }
                        }
                        if area.is_link(&areas[i], &CutType::Vertical) {
                            area.link.down.push(i);
                            areas[i].link.up.push(new_idx);
                        }
                    }
                }
            }
        }

        areas[idx] = base;
        areas.push(area);
    }
}

fn fix_room_size(areas: &mut Vec<Area>) {
    let mut rnd = thread_rng();
    for a in areas {
        a.room.w = rnd.gen_range(MIN_ROOM_SIZE..a.w - 2 * MIN_AISLE_SIZE);
        a.room.h = rnd.gen_range(MIN_ROOM_SIZE..a.h - 2 * MIN_AISLE_SIZE);
        a.room.x = rnd.gen_range(a.x + MIN_AISLE_SIZE..a.x + a.w - a.room.w - MIN_AISLE_SIZE);
        a.room.y = rnd.gen_range(a.y + MIN_AISLE_SIZE..a.y + a.h - a.room.h - MIN_AISLE_SIZE);
    }
}

fn generate_rooms(areas: &mut Vec<Area>) {
    cut_areas(areas);
    fix_room_size(areas);
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum LinkType {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Aisle {
    from: usize,
    to: usize,
    link_type: LinkType,
}

impl Aisle {
    fn new(from: usize, to: usize, link_type: LinkType) -> Aisle {
        Aisle {
            from,
            to,
            link_type,
        }
    }
}

fn create_aisle_points(a: &Room, b: &Room, link: LinkType) -> Vec<Point> {
    if link == LinkType::Up {
        return create_aisle_points(b, a, LinkType::Down);
    }
    if link == LinkType::Left {
        return create_aisle_points(b, a, LinkType::Right);
    }

    if link == LinkType::Right {
        let start_x = a.x + a.w;
        let start_y_min = a.y;
        let start_y_max = a.y + a.h;
        let end_x = b.x - 1;
        let end_y_min = b.y;
        let end_y_max = b.y + b.h;
        let turn_x = (start_x + end_x) / 2;
        let start_y = (start_y_min + start_y_max) / 2;
        let end_y = (end_y_min + end_y_max) / 2;

        let mut v = Vec::new();
        for i in start_x..=end_x {
            if i < turn_x {
                v.push(Point { x: i, y: start_y });
            } else if i == turn_x {
                for j in usize::min(start_y, end_y)..=usize::max(start_y, end_y) {
                    v.push(Point { x: i, y: j });
                }
            } else {
                v.push(Point { x: i, y: end_y });
            }
        }
        return v;
    }

    // Down
    let start_y = a.y + a.h;
    let start_x_min = a.x;
    let start_x_max = a.x + a.w;
    let end_y = b.y - 1;
    let end_x_min = b.x;
    let end_x_max = b.x + b.w;
    let turn_y = (start_y + end_y) / 2;
    let start_x = (start_x_min + start_x_max) / 2;
    let end_x = (end_x_min + end_x_max) / 2;

    let mut v = Vec::new();
    for i in start_y..=end_y {
        if i < turn_y {
            v.push(Point { x: start_x, y: i });
        } else if i == turn_y {
            for j in usize::min(start_x, end_x)..=usize::max(start_x, end_x) {
                v.push(Point { x: j, y: i });
            }
        } else {
            v.push(Point { x: end_x, y: i });
        }
    }

    v
}

fn create_aisles(areas: &[Area]) -> Vec<Point> {
    let mut rnd = thread_rng();
    let mut aisles = Vec::new();

    let mut connected_flgs = vec![false; areas.len()];
    connected_flgs[0] = true;

    while connected_flgs.iter().any(|&x| !x) {
        let mut candidates = Vec::new();

        for (idx, f) in connected_flgs.iter().enumerate() {
            if !f {
                continue;
            }
            for i in &areas[idx].link.up {
                if !connected_flgs[*i] {
                    candidates.push(Aisle::new(idx, *i, LinkType::Up));
                }
            }
            for i in &areas[idx].link.down {
                if !connected_flgs[*i] {
                    candidates.push(Aisle::new(idx, *i, LinkType::Down));
                }
            }
            for i in &areas[idx].link.left {
                if !connected_flgs[*i] {
                    candidates.push(Aisle::new(idx, *i, LinkType::Left));
                }
            }
            for i in &areas[idx].link.right {
                if !connected_flgs[*i] {
                    candidates.push(Aisle::new(idx, *i, LinkType::Right));
                }
            }
        }

        let target_i = rnd.gen_range(0..candidates.len());
        let c = candidates[target_i].clone();
        connected_flgs[c.to] = true;
        aisles.push(c);
    }

    let mut rest = Vec::new();
    for area in areas {
        for i in &area.link.up {
            let a = Aisle::new(area.idx, *i, LinkType::Up);
            if !aisles.contains(&a) {
                rest.push(a);
            }
        }
        for i in &area.link.down {
            let a = Aisle::new(area.idx, *i, LinkType::Down);
            if !aisles.contains(&a) {
                rest.push(a);
            }
        }
        for i in &area.link.left {
            let a = Aisle::new(area.idx, *i, LinkType::Left);
            if !aisles.contains(&a) {
                rest.push(a);
            }
        }
        for i in &area.link.right {
            let a = Aisle::new(area.idx, *i, LinkType::Right);
            if !aisles.contains(&a) {
                rest.push(a);
            }
        }
    }

    if !rest.is_empty() {
        let mut count = rnd.gen_range(0..usize::min(6, rest.len()));
        let mut added = Vec::new();
        while count > 0 {
            let i = rnd.gen_range(0..rest.len());
            if added.contains(&i) {
                continue;
            }
            added.push(i);
            aisles.push(rest[i].clone());
            count -= 1;
        }
    }

    let mut v = Vec::new();
    for a in aisles {
        v.extend(create_aisle_points(
            &areas[a.from].room,
            &areas[a.to].room,
            a.link_type,
        ));
    }
    v
}

pub fn generate() -> GameMap {
    let height = 50;
    let width = 100;

    let mut areas = Vec::new();
    let mut area = Area::new();
    area.h = height;
    area.w = width;
    areas.push(area);

    generate_rooms(&mut areas);
    let aisles = create_aisles(&areas);

    let mut rng = thread_rng();
    let room = &areas[rng.gen_range(0..areas.len())].room;
    let exit_pos = Position::new(
        (room.x + rng.gen_range(0..room.w)) as i32,
        (room.y + rng.gen_range(0..room.h)) as i32,
    );

    let mut map = GameMap::new(width, height, exit_pos);

    // Fill rooms
    for a in &areas {
        let room_id = a.idx as u8;
        for iy in 0..a.room.h {
            for ix in 0..a.room.w {
                let x = a.room.x + ix;
                let y = a.room.y + iy;
                map.set(x, y, MapCell { terrain: Terrain::Floor { room_id } });
            }
        }
    }

    // Fill aisles
    for p in &aisles {
        map.set(p.x, p.y, MapCell { terrain: Terrain::Aisle });
    }

    // Set exit
    map.set(
        exit_pos.x as usize,
        exit_pos.y as usize,
        MapCell { terrain: Terrain::Exit },
    );

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_returns_correct_dimensions() {
        let map = generate();
        assert_eq!(map.width, 100);
        assert_eq!(map.height, 50);
    }

    #[test]
    fn generate_has_exit() {
        let map = generate();
        assert!(map.is_exit(&map.exit_pos));
    }

    #[test]
    fn generate_has_walkable_tiles() {
        let map = generate();
        let room_positions = map.room_positions();
        assert!(!room_positions.is_empty(), "Map must have at least one room tile");
    }

    #[test]
    fn generate_exit_is_walkable_position() {
        let map = generate();
        let exit = map.exit_pos;
        // Exit should be within map bounds
        assert!(exit.x >= 0 && exit.x < map.width as i32);
        assert!(exit.y >= 0 && exit.y < map.height as i32);
    }

    #[test]
    fn generate_multiple_rooms() {
        // Run generation a few times to verify we get multiple rooms
        for _ in 0..3 {
            let map = generate();
            let positions = map.room_positions();
            // BSP should produce multiple rooms with many floor tiles
            assert!(positions.len() > 20, "Expected many room tiles, got {}", positions.len());
        }
    }

    #[test]
    fn generate_has_aisles() {
        let map = generate();
        let mut aisle_count = 0;
        for y in 0..map.height {
            for x in 0..map.width {
                if let Some(cell) = map.get(&Position::new(x as i32, y as i32)) {
                    if cell.terrain == Terrain::Aisle {
                        aisle_count += 1;
                    }
                }
            }
        }
        assert!(aisle_count > 0, "Map must have corridors connecting rooms");
    }
}
