use crate::core::types::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Wall,
    Floor { room_id: u8 },
    Aisle,
    Exit,
}

#[derive(Clone, Debug)]
pub struct MapCell {
    pub terrain: Terrain,
}

impl MapCell {
    pub fn wall() -> Self {
        MapCell {
            terrain: Terrain::Wall,
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.terrain != Terrain::Wall
    }

    pub fn is_room(&self) -> bool {
        matches!(self.terrain, Terrain::Floor { .. })
    }
}

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    cells: Vec<Vec<MapCell>>,
    pub exit_pos: Position,
}

impl GameMap {
    pub fn new(width: usize, height: usize, exit_pos: Position) -> Self {
        let cells = vec![vec![MapCell::wall(); width]; height];
        GameMap {
            width,
            height,
            cells,
            exit_pos,
        }
    }

    pub fn get(&self, pos: &Position) -> Option<&MapCell> {
        if pos.x < 0 || pos.y < 0 {
            return None;
        }
        let x = pos.x as usize;
        let y = pos.y as usize;
        self.cells.get(y).and_then(|row| row.get(x))
    }

    pub fn set(&mut self, x: usize, y: usize, cell: MapCell) {
        if y < self.height && x < self.width {
            self.cells[y][x] = cell;
        }
    }

    pub fn is_walkable(&self, pos: &Position) -> bool {
        self.get(pos).map(|c| c.is_walkable()).unwrap_or(false)
    }

    pub fn is_wall(&self, pos: &Position) -> bool {
        self.get(pos)
            .map(|c| c.terrain == Terrain::Wall)
            .unwrap_or(true)
    }

    pub fn is_exit(&self, pos: &Position) -> bool {
        self.get(pos)
            .map(|c| c.terrain == Terrain::Exit)
            .unwrap_or(false)
    }

    pub fn is_same_room(&self, a: &Position, b: &Position) -> bool {
        match (self.get(a), self.get(b)) {
            (Some(ca), Some(cb)) => match (ca.terrain, cb.terrain) {
                (Terrain::Floor { room_id: ra }, Terrain::Floor { room_id: rb }) => ra == rb,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn room_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x].is_room() {
                    positions.push(Position::new(x as i32, y as i32));
                }
            }
        }
        positions
    }

    pub fn render_char(&self, pos: &Position) -> char {
        match self.get(pos) {
            Some(cell) => match cell.terrain {
                Terrain::Wall => ' ',
                Terrain::Floor { .. } => '.',
                Terrain::Aisle => '-',
                Terrain::Exit => '+',
            },
            None => ' ',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapcell_wall() {
        let cell = MapCell::wall();
        assert_eq!(cell.terrain, Terrain::Wall);
    }

    #[test]
    fn mapcell_is_walkable() {
        assert!(!MapCell::wall().is_walkable());
        assert!(MapCell { terrain: Terrain::Floor { room_id: 0 } }.is_walkable());
        assert!(MapCell { terrain: Terrain::Aisle }.is_walkable());
        assert!(MapCell { terrain: Terrain::Exit }.is_walkable());
    }

    #[test]
    fn mapcell_is_room() {
        assert!(MapCell { terrain: Terrain::Floor { room_id: 1 } }.is_room());
        assert!(!MapCell::wall().is_room());
        assert!(!MapCell { terrain: Terrain::Aisle }.is_room());
        assert!(!MapCell { terrain: Terrain::Exit }.is_room());
    }

    #[test]
    fn gamemap_new_all_walls() {
        let map = GameMap::new(10, 5, Position::new(0, 0));
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 5);
        for y in 0..5 {
            for x in 0..10 {
                let cell = map.get(&Position::new(x, y)).unwrap();
                assert_eq!(cell.terrain, Terrain::Wall);
            }
        }
    }

    #[test]
    fn gamemap_get_out_of_bounds() {
        let map = GameMap::new(5, 5, Position::new(0, 0));
        assert!(map.get(&Position::new(-1, 0)).is_none());
        assert!(map.get(&Position::new(0, -1)).is_none());
        assert!(map.get(&Position::new(5, 0)).is_none());
        assert!(map.get(&Position::new(0, 5)).is_none());
    }

    #[test]
    fn gamemap_set_and_get() {
        let mut map = GameMap::new(5, 5, Position::new(0, 0));
        map.set(2, 3, MapCell { terrain: Terrain::Floor { room_id: 1 } });
        let cell = map.get(&Position::new(2, 3)).unwrap();
        assert_eq!(cell.terrain, Terrain::Floor { room_id: 1 });
    }

    #[test]
    fn gamemap_is_walkable_and_is_wall() {
        let mut map = GameMap::new(5, 5, Position::new(0, 0));
        let wall_pos = Position::new(0, 0);
        assert!(!map.is_walkable(&wall_pos));
        assert!(map.is_wall(&wall_pos));

        map.set(1, 1, MapCell { terrain: Terrain::Floor { room_id: 0 } });
        let floor_pos = Position::new(1, 1);
        assert!(map.is_walkable(&floor_pos));
        assert!(!map.is_wall(&floor_pos));
    }

    #[test]
    fn gamemap_is_wall_out_of_bounds_returns_true() {
        let map = GameMap::new(5, 5, Position::new(0, 0));
        assert!(map.is_wall(&Position::new(-1, 0)));
    }

    #[test]
    fn gamemap_is_exit() {
        let mut map = GameMap::new(5, 5, Position::new(2, 2));
        map.set(2, 2, MapCell { terrain: Terrain::Exit });
        assert!(map.is_exit(&Position::new(2, 2)));
        assert!(!map.is_exit(&Position::new(0, 0)));
    }

    #[test]
    fn gamemap_is_same_room() {
        let mut map = GameMap::new(5, 5, Position::new(0, 0));
        map.set(0, 0, MapCell { terrain: Terrain::Floor { room_id: 1 } });
        map.set(1, 0, MapCell { terrain: Terrain::Floor { room_id: 1 } });
        map.set(2, 0, MapCell { terrain: Terrain::Floor { room_id: 2 } });
        map.set(3, 0, MapCell { terrain: Terrain::Aisle });

        let a = Position::new(0, 0);
        let b = Position::new(1, 0);
        let c = Position::new(2, 0);
        let d = Position::new(3, 0);

        assert!(map.is_same_room(&a, &b));   // same room_id
        assert!(!map.is_same_room(&a, &c));  // different room_id
        assert!(!map.is_same_room(&a, &d));  // aisle is not a room
    }

    #[test]
    fn gamemap_room_positions() {
        let mut map = GameMap::new(3, 3, Position::new(0, 0));
        map.set(0, 0, MapCell { terrain: Terrain::Floor { room_id: 1 } });
        map.set(1, 1, MapCell { terrain: Terrain::Aisle });
        map.set(2, 2, MapCell { terrain: Terrain::Floor { room_id: 2 } });

        let positions = map.room_positions();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&Position::new(0, 0)));
        assert!(positions.contains(&Position::new(2, 2)));
    }

    #[test]
    fn gamemap_render_char() {
        let mut map = GameMap::new(5, 5, Position::new(0, 0));
        map.set(0, 0, MapCell { terrain: Terrain::Floor { room_id: 0 } });
        map.set(1, 0, MapCell { terrain: Terrain::Aisle });
        map.set(2, 0, MapCell { terrain: Terrain::Exit });

        assert_eq!(map.render_char(&Position::new(0, 0)), '.');
        assert_eq!(map.render_char(&Position::new(1, 0)), '-');
        assert_eq!(map.render_char(&Position::new(2, 0)), '+');
        assert_eq!(map.render_char(&Position::new(3, 0)), ' '); // wall
        assert_eq!(map.render_char(&Position::new(-1, 0)), ' '); // out of bounds
    }
}
