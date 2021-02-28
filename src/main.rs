extern crate termion;

use ayz::map;
use std::io::{stdin, stdout, Read, Write};
use rand::{thread_rng, Rng};
use termion::raw::IntoRawMode;

trait Cell {
    fn is_room(self) -> bool;
}

impl Cell for char {
    fn is_room(self) -> bool {
        self.is_alphabetic()
    }
}

struct Dungeon {
    pub floor: usize,
    pub turn: usize,

    pub player: Player,
    pub monsters: Vec<Monster>,
    pub status: String,

    pub map: Map,
}

impl Dungeon {
    fn can_move(&self, pos: &Position) -> bool {
        if self.player.pos.x == pos.x && self.player.pos.y == pos.y {
            return false;
        }

        for m in self.monsters.iter() {
            if m.pos.x == pos.x && m.pos.y == pos.y {
                return false;
            }
        }
        return !self.map.is_wall(pos);
    }

    fn move_monsters(&mut self) {
        let mut rng = thread_rng();
        let dps = vec![
            Position { x: -1, y: -1 },
            Position { x: 0, y: -1 },
            Position { x: 1, y: -1 },
            Position { x: -1, y: 0 },
            Position { x: 1, y: 0 },
            Position { x: -1, y: 1 },
            Position { x: 0, y: 1 },
            Position { x: 1, y: 1 },
        ];

        let mut new_pos = Vec::new();
        for m in self.monsters.iter() {
            let mut v = Vec::new();

            for dp in dps.iter() {
                if self.can_move(&m.pos.plus(dp)) {
                    v.push(dp);
                }
            }

            if v.is_empty() {
                new_pos.push(m.pos);
            } else {
                let dp = v[rng.gen_range(0..v.len())];
                new_pos.push(m.pos.plus(dp));
            }
        }

        for (i, m) in self.monsters.iter_mut().enumerate() {
            m.pos = new_pos[i];
        }
    }
}

struct Map {
    pub cells: Vec<String>,
}

impl Map {
    fn get_cell(&self, pos: &Position) -> Option<char> {
        let ln = self.cells.get(pos.y as usize);
        if ln == None {
            return None;
        }

        return ln.unwrap().chars().nth(pos.x as usize);
    }

    fn is_wall(&self, pos: &Position) -> bool {
        return self.get_cell(pos) == Some(' ');
    }
    
    //fn is_room(&self, pos: &Position) -> bool {
    //    return match self.get_cell(pos) {
    //        None => false,
    //        Some(c) => c.is_ascii(),
    //    };
    //}
}

#[derive(Clone, Copy)]
struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    fn plus(&self, pos: &Position) -> Position {
        Position {
            x: self.x + pos.x,
            y: self.y + pos.y,
        }
    }
}

const U: Position = Position { x: 0, y: -1 };
const D: Position = Position { x: 0, y: 1 };
const L: Position = Position { x: -1, y: 0 };
const R: Position = Position { x: 1, y: 0 };

const UR: Position = Position { x: 1, y: -1 };
const UL: Position = Position { x: -1, y: -1 };
const DR: Position = Position { x: 1, y: 1 };
const DL: Position = Position { x: -1, y: 1 };

struct Player {
    pub level: i32,

    pub symbol: String,
    pub hp: i32,
    pub power: i32,

    pub pos: Position,
    pub direction: Position,
}

struct Monster {
    pub symbol: String,
    pub hp: i32,
    pub power: i32,

    pub pos: Position,
}

macro_rules! write_game {
    ($stdout:expr, $dungeon:expr) =>{
        write!($stdout, "{}", termion::clear::All).unwrap();

        write!($stdout, "{}> p({}, {}). {}", termion::cursor::Goto(1, 1), $dungeon.player.pos.x, $dungeon.player.pos.y, $dungeon.status).unwrap();

        for (i, val) in $dungeon.map.cells.iter().enumerate() {
            for (j, v) in val.chars().enumerate() {
                let c = if v.is_room() {
                    '.'
                } else {
                    v
                };
                write!($stdout, "{}{}", termion::cursor::Goto(1 + j as u16, 2 + i as u16), c).unwrap();
            }
        }

        let player = &$dungeon.player;
        write!($stdout, "{}{}", termion::cursor::Goto(1 + player.pos.x as u16, 2 + player.pos.y as u16), player.symbol).unwrap();

        for i in $dungeon.monsters.iter() {
            write!($stdout, "{}{}", termion::cursor::Goto(1 + i.pos.x as u16, 2 + i.pos.y as u16), i.symbol).unwrap();
        }
        $stdout.flush().unwrap();
    }
}

fn calc_spawn_pos(map: &Map) -> Position {
    let mut v = Vec::new();
    for (i, s) in map.cells.iter().enumerate() {
        for (j, c) in s.chars().enumerate() {
            if c.is_room() {
                v.push((i, j));
            }
        }
    }

    if v.is_empty() {
        Position {
            x: 0,
            y: 0,
        }
    } else {
        let mut rng = thread_rng();
        let p = v[rng.gen_range(0..v.len())];

        Position {
            x: p.1 as i32,
            y: p.0 as i32,
        }
    }
}

fn main() {
    /*
    let map = Map {
        cells: vec![
            String::from("##########################################################"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa########bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa########bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa----####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-####bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa###-----bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa########bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("#aaaaaaaaaaaaaaaaaaaaaa########bbbbbbbbbbbbbbbbbbbbbbbbbb#"),
            String::from("##########################################################"),
        ],
    };
    */
    let map = Map {
        cells: map::gen(),
    };

    let pos = calc_spawn_pos(&map);

    let player = Player {
        level: 1,

        symbol: String::from("@"),
        hp: 15,
        power: 8,

        pos,
        direction: D,
    };

    let monsters = vec![
        Monster {
            symbol: String::from("M"),
            hp: 10,
            power: 1,

            pos: calc_spawn_pos(&map),
        }
    ];

    let mut dungeon = Dungeon {
        floor: 1,
        turn: 0,

        player,
        monsters,
        status: String::from("start"),

        map,
    };

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    write_game!(stdout, dungeon);

    let mut bytes = stdin.bytes();
    loop {
        let b = bytes.next().unwrap().unwrap();

        let mut next_pos = dungeon.player.pos.clone();
        match b {
            b'q' => {
                break;
            }
            b'h' => {
                next_pos = next_pos.plus(&L);
                dungeon.player.direction = L;
            }
            b'j' => {
                next_pos = next_pos.plus(&D);
                dungeon.player.direction = D;
            }
            b'k' => {
                next_pos = next_pos.plus(&U);
                dungeon.player.direction = U;
            }
            b'l' => {
                next_pos = next_pos.plus(&R);
                dungeon.player.direction = R;
            }
            b'y' => {
                next_pos = next_pos.plus(&UL);
                dungeon.player.direction = UL;
            }
            b'u' => {
                next_pos = next_pos.plus(&UR);
                dungeon.player.direction = UR;
            }
            b'n' => {
                next_pos = next_pos.plus(&DL);
                dungeon.player.direction = DL;
            }
            b'm' => {
                next_pos = next_pos.plus(&DR);
                dungeon.player.direction = DR;
            }
            a => {
                dungeon.status = format!("{}", a);
            }
        }

        if dungeon.can_move(&next_pos) {
            dungeon.player.pos = next_pos;
            dungeon.status = String::from("move");

            dungeon.move_monsters();
        } else {
            dungeon.status = String::from("Failed to move");
        }

        write_game!(stdout, dungeon);
    }

    write!(stdout, "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
