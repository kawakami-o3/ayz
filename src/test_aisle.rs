
#[derive(Debug)]
struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,

    c: char,
}

impl Room {
    fn new(x: usize, y: usize, w: usize, h: usize, c: char) -> Room {
        Room { x, y, w, h, c }
    }
}

#[derive(PartialEq)]
enum LinkType {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone)]
struct Point {
    x: usize,
    y: usize,
}

fn create_aisle(a: &Room, b: &Room, link: LinkType) -> Vec<Point> {

    if link == LinkType::UP {
        return create_aisle(b, a, LinkType::DOWN);
    }
    if link == LinkType::LEFT {
        return create_aisle(b, a, LinkType::RIGHT);
    }


    // RIGHT: a.right => b.left
    if link == LinkType::RIGHT {

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
                v.push(Point { x: i, y: start_y});
            } else if i == turn_x {
                for j in usize::min(start_y, end_y)..=usize::max(start_y, end_y) {
                    v.push(Point { x: i, y: j });
                }
            } else { // i > turn_x
                v.push(Point { x: i, y: end_y});
            }
        }
        return v;
    }

    // DOWN: a.down => b.up
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
            v.push(Point { x: start_x, y: i});
        } else if i == turn_y {
            for j in usize::min(start_x, end_x)..=usize::max(start_x, end_x) {
                v.push(Point { x: j, y: i });
            }
        } else { // i > turn_x
            v.push(Point { x: end_x, y: i});
        }
    }

    return v;

}


fn main() {

    let h = 50;
    let w = 100;

    let a = Room::new(45, 20, 10, 10, '1');
    let b = Room::new(80, 39, 10, 10, '2');
    let c = Room::new(5, 5, 10, 10, '3');
    let d = Room::new(75, 10, 10, 10, '4');
    let e = Room::new(15, 35, 10, 10, '5');

    let mut aisle = Vec::new();
    aisle = [aisle, create_aisle(&e, &a, LinkType::RIGHT)].concat();
    aisle = [aisle, create_aisle(&c, &a, LinkType::DOWN)].concat();
    aisle = [aisle, create_aisle(&d, &a, LinkType::LEFT)].concat();
    aisle = [aisle, create_aisle(&b, &a, LinkType::UP)].concat();

    let rooms = vec![a, b, c, d, e];

    let mut output = Vec::new();
    for _j in 0..h {
        let mut ln = Vec::new();
        for _i in 0..w {
            ln.push(' ');
        }
        output.push(ln);
    }

    for r in rooms {
        for j in 0..r.h {
            for i in 0..r.w {
                output[r.y + j][r.x + i] = r.c;
            }
        }
    }

    for i in aisle {
        println!("{:?}", i);
        output[i.y][i.x] = '-';
        //output[i.x][i.y] = '-';
    }


    for j in output {
        for i in j {
            print!("{}", i);
        }
        println!();
    }
}
