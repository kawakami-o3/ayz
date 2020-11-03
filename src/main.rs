extern crate termion;

//use termion::color;
use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;

struct Dungeon {
    pub floor: usize,
    pub turn: usize,

    pub player: Player,
}

struct Player {
    pub level: i32,
    pub hp: i32,
    pub power: i32,
}

macro_rules! writeGame {
    ($stdout:expr, $x:expr, $y:expr, $status:expr) =>{
        let player = "@";
        write!($stdout, "{}", termion::clear::All).unwrap();
        write!($stdout, "{}> {}", termion::cursor::Goto(1, 1), $status).unwrap();
        write!($stdout, "{}{}", termion::cursor::Goto($x, $y), player).unwrap();
        $stdout.flush().unwrap();
    }
}

fn main() {
    let player = Player {
        level: 1,
        hp: 15,
        power: 8,
    };

    let mut dungeon = Dungeon {
        floor: 1,
        turn: 0,

        player: player,
    };

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let mut x = 20;
    let mut y = 10;

    let mut status = String::from("");

    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    writeGame!(stdout, x, y, status);

    let mut bytes = stdin.bytes();
    loop {
        let b = bytes.next().unwrap().unwrap();

        match b {
            b'q' => {
                break;
            }
            b'h' => {
                x -= 1;
            }
            b'j' => {
                y += 1;
            }
            b'k' => {
                y -= 1;
            }
            b'l' => {
                x += 1;
            }
            a => {
                status = format!("{}", a);
            }
        }

        writeGame!(stdout, x, y, status);
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
