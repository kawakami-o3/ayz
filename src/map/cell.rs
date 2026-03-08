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
