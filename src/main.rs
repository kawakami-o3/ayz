extern crate termion;

//use termion::color;
use std::io::{stdin, stdout, Read, Write};
use rand::{thread_rng, Rng};
use termion::raw::IntoRawMode;


struct Dungeon {
    pub floor: usize,
    pub turn: usize,

    pub player: Player,
    pub status: String,

    pub map: Map,
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

#[derive(Clone)]
struct Position {
    pub x: u16,
    pub y: u16,
}

struct Player {
    pub level: i32,
    pub hp: i32,
    pub power: i32,

    pub pos: Position,
}

macro_rules! write_game {
    ($stdout:expr, $dungeon:expr) =>{
        let player = "@";

        write!($stdout, "{}", termion::clear::All).unwrap();

        write!($stdout, "{}> p({}, {}). {}", termion::cursor::Goto(1, 1), $dungeon.player.pos.x, $dungeon.player.pos.y, $dungeon.status).unwrap();

        for (i, val) in $dungeon.map.cells.iter().enumerate() {
            write!($stdout, "{}{}", termion::cursor::Goto(1, 2 + i as u16), val).unwrap();
        }

        //write!($stdout, "{}{}", termion::cursor::Goto(1 + $dungeon.player.x, 2 + $dungeon.player.y), player).unwrap();
        write!($stdout, "{}{}", termion::cursor::Goto(1 + $dungeon.player.pos.x, 2 + $dungeon.player.pos.y), player).unwrap();
        $stdout.flush().unwrap();
    }
}

fn calc_spawn_pos(map: &Map) -> Position {
    let mut v = Vec::new();
    for (i, s) in map.cells.iter().enumerate() {
        //for (j, s) in s.iter().enumerate() {
        for (j, c) in s.chars().enumerate() {
            if c == '.' {
                v.push((i as u16, j as u16));
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
            x: p.1,
            y: p.0,
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
        hp: 15,
        power: 8,

        pos,
    };


    let mut dungeon = Dungeon {
        floor: 1,
        turn: 0,

        player,
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

        if dungeon.map.is_wall(&next_pos) {
            dungeon.status = String::from("wall");
        } else {
            dungeon.player.pos = next_pos;
            dungeon.status = String::from("move");
        }

        write_game!(stdout, dungeon);
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
