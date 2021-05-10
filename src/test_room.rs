//use rand::prelude::*;
use rand::Rng;

// 部屋の仕様
// 適当に分割した区画を作成し、その中に部屋を生成する
// 通路は部屋と1マス以上距離をおいて生成される
// 部屋の最小サイズは一辺が 3 マスとする
// 区画内に通路をひけるように端に 2 マスの猶予を残しておく.
// (厳密にはどちらかの区画に猶予があれば良いので、もっと減らせるが簡単のためやらない)
// よって、区画の最小サイズは 3 + 2 * 2 = 7 となる.
//
// コードを拡張しながら作っているため名前の対応が多少変だが、
// 区画をRoom、部屋をAreaとする. 本実装では逆にした方がいいだろう.

#[derive(Clone, Debug)]
struct Area {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Area {
    fn new() -> Area {
        Area {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }
    }
}

//#[derive(Clone, Debug, Copy)]
#[derive(Clone, Debug)]
struct Room {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub idx: usize,

    pub link: Link,
    pub area: Area,
}

//#[derive(Clone, Debug, Copy)]
#[derive(Clone, Debug)]
struct Link {
    pub up: Vec<usize>,
    pub down: Vec<usize>,
    pub left: Vec<usize>,
    pub right: Vec<usize>,
}

impl Room {
    fn new() -> Room {
        Room {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            idx: 0,
            link: Link {
                up: Vec::new(),
                down: Vec::new(),
                left: Vec::new(),
                right: Vec::new(),
            },
            area: Area::new(),
        }
    }
}

#[derive(PartialEq)]
enum CutType {
    Vertical,
    Horizontal,
}

fn calc_total(rooms: &Vec<Room>) -> usize {
    let mut total = 0;
    for r in rooms {
        total += r.h * r.w;
    }
    return total;
}

fn choose(rooms: &Vec<Room>) -> usize {
    let mut rnd = rand::thread_rng();
    let target = rnd.gen_range(0..calc_total(&rooms));

    let mut sum = 0;
    for (i, r) in rooms.iter().enumerate() {
        sum += r.h * r.w;
        if target < sum {
            return i;
        }
    }

    return 0;
}

fn calc_cut_size(size: usize) -> usize {
    // TODO 分割はおなじ大きさで
    return size / 2;
}

// cut_area
fn cut_rooms(rooms: &mut Vec<Room>) {
    let mut rnd = rand::thread_rng();
    for _i in 0..10 {
        // TODO 分割できない場合の処理. 対象を選んで、どれも当てはまらなかったらbreakという感じ.
        //      境界に1セルは使うということに注意. 両方含めると2セル.

        // TODO どの部屋を分割するか
        //let idx = 0;
        //let idx = rnd.gen_range(0..rooms.len());
        let idx = choose(rooms);
        //if idx < 0 { break; }
        println!("idx:{} len:{}", idx, rooms.len());

        let cut_type = &vec![CutType::Horizontal, CutType::Vertical][rnd.gen_range(0..2)];
        //let cut_type = CutType::Vertical;

        let new_idx = rooms.len();

        if rooms[idx].w < 14 || rooms[idx].h < 14 {
            // 縦横両方が不足ならとしたいが、その場合は分割方向にも注意が必要.
            // あまって短い方で切らないように.
            continue;
        }

        let mut base = rooms[idx].clone();
        let mut room = Room::new();
        room.idx = new_idx;

        // 分割して位置、大きさを設定

        match cut_type {
            CutType::Horizontal => {
                let new_size = calc_cut_size(base.h);

                room.x = base.x;
                room.y = base.y + new_size;
                room.w = base.w;
                room.h = base.h - new_size;

                base.h = new_size;
            }
            CutType::Vertical => {
                let new_size = calc_cut_size(base.w);

                room.x = base.x + new_size;
                room.y = base.y;
                room.w = base.w - new_size;
                room.h = base.h;

                base.w = new_size;
            }
        }

        // リンク関係を修正
        match cut_type {
            CutType::Horizontal => {
                for i in base.link.down {
                    let mut target = None;
                    let mut link = rooms[i].link.clone();
                    for (ii, j) in link.up.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        room.link.down.push(rooms[i].idx);
                        link.up.remove(target_idx);
                        link.up.push(new_idx);
                        rooms[i].link = link;
                    }
                }

                base.link.down = vec![new_idx];
                room.link.up = vec![idx];

                for i in &base.link.right {
                    rooms[*i].link.left.push(new_idx);
                }
                for i in &base.link.left {
                    rooms[*i].link.right.push(new_idx);
                }
            }
            CutType::Vertical => {
                for i in base.link.right {
                    let mut target = None;
                    let mut link = rooms[i].link.clone();
                    for (ii, j) in link.left.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        room.link.right.push(rooms[i].idx);
                        link.left.remove(target_idx);
                        link.left.push(new_idx);
                        rooms[i].link = link;
                    }
                }

                base.link.right = vec![new_idx];
                room.link.left = vec![idx];

                for i in &base.link.up {
                    rooms[*i].link.left.push(new_idx);
                }
                for i in &base.link.down {
                    rooms[*i].link.right.push(new_idx);
                }
            }
        }

        rooms[idx] = base;
        rooms.push(room);
    }
}

// fix_room
fn resize_rooms(rooms: &mut Vec<Room>) {
    let mut rnd = rand::thread_rng();

    let edge = 2;
    for r in rooms {
        println!("wh: {}, {}", r.w, r.h);
        r.area.w = rnd.gen_range(3..r.w - 2 * edge);
        r.area.h = rnd.gen_range(3..r.h - 2 * edge);
        println!("x:{}..{}", r.x + edge, r.x + r.w - r.area.w - edge);
        r.area.x = rnd.gen_range(r.x + edge..r.x + r.w - r.area.w - edge);
        println!("y:{}..{}", r.y + edge, r.y + r.h - r.area.h - edge);
        r.area.y = rnd.gen_range(r.y + edge..r.y + r.h - r.area.h - edge);
    }
}

//fn gen(rooms: &mut Vec<Room>, links: &mut Vec<Link>) {
fn gen(rooms: &mut Vec<Room>) {
    cut_rooms(rooms);
    resize_rooms(rooms);
}

fn main() {
    let height = 100;
    let width = 200;
    let mut rooms = Vec::new();

    let mut r = Room::new();
    r.h = height;
    r.w = width;

    rooms.push(r);

    //let mut links = Vec::new();
    //gen(&mut rooms, &mut links);

    gen(&mut rooms);

    let mut output = Vec::new();

    //for _i in 0..height {
    //    output.push("*".repeat(width));
    //}
    //
    for _i in 0..height {
        let mut row = Vec::new();
        for _j in 0..width {
            //output.push("*".repeat(width));
            row.push('*');
        }
        output.push(row);
    }

    for r in rooms {
        println!("{:?}", r);
        for iy in 0..r.area.h {
            for ix in 0..r.area.w {
                let x = r.area.x + ix;
                let y = r.area.y + iy;
                output[y][x] = format!("{}", r.idx).chars().next().unwrap();
            }
        }
    }

    //println!("{:?}", links);
    for i in 0..height {
        for j in 0..width {
            print!("{}", output[i][j]);
        }
        println!();
    }
}

/*
fn main() {
    let x = 100; // col
    let y = 50;  // row



    let mut map = Vec::new();
    for _i in 0..y {

        let mut ln = Vec::new();
        for _j in 0..x {
            ln.push(0);
        }

        map.push(ln);
    }

    for ln in map {
        for i in ln {
            print!("{}", i);
        }
        println!();
    }
}
*/
