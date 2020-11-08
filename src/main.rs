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

struct Player {
    pub level: i32,
    pub hp: i32,
    pub power: i32,

    pub x: u16,
    pub y: u16,
}

macro_rules! write_game {
    ($stdout:expr, $dungeon:expr) =>{
        let player = "@";

        write!($stdout, "{}", termion::clear::All).unwrap();

        write!($stdout, "{}> p({}, {}). {}", termion::cursor::Goto(1, 1), $dungeon.player.x, $dungeon.player.y, $dungeon.status).unwrap();

        for (i, val) in $dungeon.map.cells.iter().enumerate() {
            write!($stdout, "{}{}", termion::cursor::Goto(1, 2 + i as u16), val).unwrap();
        }

        //write!($stdout, "{}{}", termion::cursor::Goto(1 + $dungeon.player.x, 2 + $dungeon.player.y), player).unwrap();
        write!($stdout, "{}{}", termion::cursor::Goto(1 + $dungeon.player.x, 2 + $dungeon.player.y), player).unwrap();
        $stdout.flush().unwrap();
    }
}

fn calc_spawn_pos(map: &Map) -> (u16, u16) {
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
        (0, 0)
    } else {
        let mut rng = thread_rng();
        v[rng.gen_range(0, v.len())]
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

        x: pos.1,
        y: pos.0,
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

        match b {
            b'q' => {
                break;
            }
            b'h' => {
                dungeon.player.x -= 1;
            }
            b'j' => {
                dungeon.player.y += 1;
            }
            b'k' => {
                dungeon.player.y -= 1;
            }
            b'l' => {
                dungeon.player.x += 1;
            }
            a => {
                dungeon.status = format!("{}", a);
            }
        }

        write_game!(stdout, dungeon);
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
