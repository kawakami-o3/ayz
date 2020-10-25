extern crate termion;

//use termion::color;
use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let s = "@";

    let mut x = 20;
    let mut y = 10;

    write!(
        stdout,
        "{}{}> start{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Goto(x, y),
        s
    )
    .unwrap();
    stdout.flush().unwrap();

    let mut bytes = stdin.bytes();
    loop {
        let b = bytes.next().unwrap().unwrap();

        match b {
            b'q' => {
                return;
            }
            b'h' => {
                write!(stdout, "{} ", termion::cursor::Goto(x, y)).unwrap();
                x -= 1;
                write!(stdout, "{}{}", termion::cursor::Goto(x, y), s).unwrap();
            }
            b'j' => {
                write!(stdout, "{} ", termion::cursor::Goto(x, y)).unwrap();
                y += 1;
                write!(stdout, "{}{}", termion::cursor::Goto(x, y), s).unwrap();
            }
            b'k' => {
                write!(stdout, "{} ", termion::cursor::Goto(x, y)).unwrap();
                y -= 1;
                write!(stdout, "{}{}", termion::cursor::Goto(x, y), s).unwrap();
            }
            b'l' => {
                write!(stdout, "{} ", termion::cursor::Goto(x, y)).unwrap();
                x += 1;
                write!(stdout, "{}{}", termion::cursor::Goto(x, y), s).unwrap();
            }
            a => {
                write!(
                    stdout,
                    "{}{}> {}",
                    termion::clear::All,
                    termion::cursor::Goto(1, 1),
                    a
                )
                .unwrap();
                write!(stdout, "{}{}", termion::cursor::Goto(x, y), s).unwrap();
            }
        }

        stdout.flush().unwrap();
    }
}
