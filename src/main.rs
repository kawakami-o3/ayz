extern crate termion;

//use termion::color;
use std::io::{stdin, stdout, Read, Write};
use rand::{thread_rng, Rng};
use termion::raw::IntoRawMode;


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
                let dp = v[rng.gen_range(0, v.len())];
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
    fn is_wall(&self, pos: &Position) -> bool {
        let ln = self.cells.get(pos.y as usize); //[pos.x as usize].chars().nth(pos.y as usize);
        if ln == None {
            panic!();
        }

        let c = ln.unwrap().chars().nth(pos.x as usize);
        if c == None {
            panic!();
        }

        return c == Some('#');
    }
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

struct Player {
    pub level: i32,

    pub symbol: String,
    pub hp: i32,
    pub power: i32,

    pub pos: Position,
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
            write!($stdout, "{}{}", termion::cursor::Goto(1, 2 + i as u16), val).unwrap();
        }

        //write!($stdout, "{}{}", termion::cursor::Goto(1 + $dungeon.player.x, 2 + $dungeon.player.y), player).unwrap();
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
        //for (j, s) in s.iter().enumerate() {
        for (j, c) in s.chars().enumerate() {
            if c == '.' {
                //v.push((i as u16, j as u16));
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
        let p = v[rng.gen_range(0, v.len())];

        Position {
            x: p.1 as i32,
            y: p.0 as i32,
        }
    }
}

fn main() {
    let map = Map {
        cells: vec![
            String::from("##########################################################"),
            String::from("#......................########..........................#"),
            String::from("#......................########..........................#"),
            String::from("#......................----####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-####..........................#"),
            String::from("#......................###-----..........................#"),
            String::from("#......................########..........................#"),
            String::from("#......................########..........................#"),
            String::from("##########################################################"),
        ],
    };

    let pos = calc_spawn_pos(&map);

    let player = Player {
        level: 1,

        symbol: String::from("@"),
        hp: 15,
        power: 8,

        pos,
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
                next_pos.x -= 1;
            }
            b'j' => {
                next_pos.y += 1;
            }
            b'k' => {
                next_pos.y -= 1;
            }
            b'l' => {
                next_pos.x += 1;
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
