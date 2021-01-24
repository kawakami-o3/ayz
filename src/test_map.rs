use rand::Rng;

const MIN_ROOM_SIZE: usize = 3;
const MIN_AISLE_SIZE: usize = 2;
const MIN_CUT_SIZE: usize = 2 * (MIN_ROOM_SIZE + MIN_AISLE_SIZE * 2); // 部屋は最小 3, 通路の余白 2x2 とすると分割前の最小サイズは 14.
const CUT_TRIAL: usize = 10;

#[derive(Clone, Debug)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,

    idx: usize,
}

impl Room {
    fn new() -> Self {
        Self { x: 0, y: 0, w: 0, h: 0, idx: 0 }
    }
}

// 区画切り分け時にはちゃんと隣接しているかチェックする必要がある
#[derive(Clone, Debug)]
struct Link {
    up: Vec<usize>,
    down: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
}

impl Link {
    fn new() -> Link {
        Link {
            up: Vec::new(),
            down: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct Area {
    x: usize,
    y: usize,
    w: usize,
    h: usize,

    idx: usize,

    link: Link,
    room: Room,
}

impl Area {
    fn new() -> Area {
        Area {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            idx: 0,
            link: Link::new(),
            room: Room::new(),
        }
    }

    fn is_link(&self, target: & Area, cut_type: CutType) -> bool {
        // TODO 通路分の余白は隣接判定に含めたくない
        match cut_type {
            CutType::Horizontal => {
                ! (self.y < (target.y + target.h) || target.y < (self.y + self.h))
            }
            CutType::Vertical => {
                ! (self.x < (target.x + target.w) || target.x < (self.x + self.w))
            }
        }

    }
}

fn calc_weight(a: & Area) -> usize {
    a.w * a.h
}

fn choose(areas: & Vec<Area>) -> usize {
    //let mut rnd = rand::thread_rng();
    //let total_weight = areas.iter().map(calc_weight).sum();
    //let target = rnd.gen_range(0..total_weight);

    //let mut sum = 0;
    //for (i, a) in areas.iter().enumerate() {
    //    sum += calc_weight(a);
    //    if target < sum {
    //        return i;
    //    }
    //}

    let mut rnd = rand::thread_rng();
    let mut total_weight = 0;
    for a in areas {
        if a.w >= MIN_CUT_SIZE || a.h >= MIN_CUT_SIZE {
            total_weight += calc_weight(a);
        }
    }
    let target = rnd.gen_range(0..total_weight);

    let mut sum = 0;
    for (i, a) in areas.iter().enumerate() {
        if a.w >= MIN_CUT_SIZE || a.h >= MIN_CUT_SIZE {
            sum += calc_weight(a);
            if target < sum {
                return i;
            }
        }
    }
    return 0;
}

#[derive(PartialEq)]
enum CutType {
    Vertical,
    Horizontal,
}

// TODO 分割サイズ
fn calc_cut_size(size: usize) -> usize {
    size / 2
}

fn cut_areas(areas: &mut Vec<Area>) {
    let mut rnd = rand::thread_rng();

    // TODO 分割回数
    for _i in 0..CUT_TRIAL {
        let idx = choose(areas);

        // TODO リファクタリング. chooseの中でもサイズのチェックを行っている.
        if areas[idx].w < MIN_CUT_SIZE && areas[idx].h < MIN_CUT_SIZE {
            continue;
        }

        let mut base = areas[idx].clone();

        // TODO 対象区画のサイズに応じて分割タイプを変更
        let mut cut_type_list = Vec::new();
        if base.w >= MIN_CUT_SIZE {
            cut_type_list.push(CutType::Vertical);
        }
        if base.h >= MIN_CUT_SIZE {
            cut_type_list.push(CutType::Horizontal);
        }
        let cut_type = &cut_type_list[rnd.gen_range(0..cut_type_list.len())];

        let new_idx = areas.len();


        let mut area = Area::new();
        area.idx = new_idx;

        // サイズ修正
        match cut_type {
            CutType::Horizontal => {
                let new_size = calc_cut_size(base.h);

                area.x = base.x;
                area.y = base.y + new_size;
                area.w = base.w;
                area.h = base.h - new_size;

                base.h = new_size;
            }
            CutType::Vertical => {
                let new_size = calc_cut_size(base.w);

                area.x = base.x + new_size;
                area.y = base.y;
                area.w = base.w - new_size;
                area.h = base.h;

                base.w = new_size;
            }
        }

        // 隣接対象修正
        // TODO 本当に隣接しているかをちゃんとみる必要がある
        // TODO 隣接情報は最後に一括で構築するほうが楽かも
        match cut_type {
            CutType::Horizontal => {
                for i in base.link.down {
                    let mut target = None;
                    let mut link = areas[i].link.clone();
                    for (ii, j) in link.up.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        area.link.down.push(areas[i].idx);
                        link.up.remove(target_idx);
                        link.up.push(new_idx);
                        areas[i].link = link;
                    }
                }

                base.link.down = vec![new_idx];
                area.link.up = vec![idx];

                // 右隣接
                {
                    let old_link = base.link.right.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], CutType::Horizontal) {
                            // TODO リファクタリング vec.remove_item
                            {
                                let ii = base.link.right.iter().position(|x| *x == i);
                                if let Some(target_i) = ii {
                                    base.link.right.remove(target_i);
                                }
                            }
                            {
                                let ii = areas[i].link.left.iter().position(|x| *x == idx);
                                if let Some(target_i) = ii {
                                    areas[i].link.left.remove(target_i);
                                }
                            }
                        }
                        if area.is_link(&areas[i], CutType::Horizontal) {
                            area.link.right.push(i);
                            areas[i].link.left.push(new_idx);
                        }
                    }
                }

                // 左隣接
                {
                    let old_link = base.link.left.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], CutType::Horizontal) {
                            //base.link.left.remove_itemp(i);
                            //areas[i].link.right.remove_item(idx);

                            // TODO リファクタリング vec.remove_item
                            {
                                let ii = base.link.left.iter().position(|x| *x == i);
                                if let Some(target_i) = ii {
                                    base.link.left.remove(target_i);
                                }
                            }
                            {
                                let ii = areas[i].link.right.iter().position(|x| *x == idx);
                                if let Some(target_i) = ii {
                                    areas[i].link.right.remove(target_i);
                                }
                            }
                        }
                        if area.is_link(&areas[i], CutType::Horizontal) {
                            area.link.left.push(i);
                            areas[i].link.right.push(new_idx);
                        }
                    }
                }
            }
            CutType::Vertical => {

                for i in base.link.right {
                    let mut target = None;
                    let mut link = areas[i].link.clone();
                    for (ii, j) in link.left.iter().enumerate() {
                        if *j == idx {
                            target = Some(ii);
                        }
                    }
                    if let Some(target_idx) = target {
                        area.link.right.push(areas[i].idx);
                        link.left.remove(target_idx);
                        link.left.push(new_idx);
                        areas[i].link = link;
                    }
                }

                base.link.right = vec![new_idx];
                area.link.left = vec![idx];

                // 上隣接
                {
                    let old_link = base.link.up.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], CutType::Vertical) {
                            //base.link.up.remove_item(i);
                            //areas[i].link.down.remove_item(idx);
                            // TODO リファクタリング vec.remove_item
                            {
                                let ii = base.link.up.iter().position(|x| *x == i);
                                if let Some(target_i) = ii {
                                    base.link.up.remove(target_i);
                                }
                            }
                            {
                                let ii = areas[i].link.down.iter().position(|x| *x == idx);
                                if let Some(target_i) = ii {
                                    areas[i].link.down.remove(target_i);
                                }
                            }
                        }
                        if area.is_link(&areas[i], CutType::Vertical) {
                            area.link.up.push(i);
                            areas[i].link.down.push(new_idx);
                        }
                    }
                }

                // 下隣接
                {
                    let old_link = base.link.down.clone();
                    for i in old_link {
                        if !base.is_link(&areas[i], CutType::Vertical) {
                            //base.link.down.remove(i);
                            //areas[i].link.up.remove(idx);
                        
                            // TODO リファクタリング vec.remove_item
                            {
                                let ii = base.link.down.iter().position(|x| *x == i);
                                if let Some(target_i) = ii {
                                    base.link.down.remove(target_i);
                                }
                            }
                            {
                                let ii = areas[i].link.up.iter().position(|x| *x == idx);
                                if let Some(target_i) = ii {
                                    areas[i].link.up.remove(target_i);
                                }
                            }
                        }
                        if area.is_link(&areas[i], CutType::Vertical) {
                            area.link.down.push(i);
                            areas[i].link.up.push(new_idx);
                        }
                    }
                }
            }
        }

        areas[idx] = base;
        areas.push(area);
    }
}

fn fix_room_size(areas: &mut Vec<Area>) {
    let mut rnd = rand::thread_rng();

    for a in areas {
        println!("{:?}", a);
        a.room.w = rnd.gen_range(MIN_ROOM_SIZE .. a.w - 2 * MIN_AISLE_SIZE);
        a.room.h = rnd.gen_range(MIN_ROOM_SIZE .. a.h - 2 * MIN_AISLE_SIZE);
        a.room.x = rnd.gen_range(a.x + MIN_AISLE_SIZE .. a.x + a.w - a.room.w - MIN_AISLE_SIZE);
        a.room.y = rnd.gen_range(a.y + MIN_AISLE_SIZE .. a.y + a.h - a.room.h - MIN_AISLE_SIZE);
    }
}

fn generate_rooms(areas: &mut Vec<Area>) {
    cut_areas(areas);
    fix_room_size(areas);
}

#[derive(PartialEq, Clone, Debug)]
enum LinkType {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Clone, Debug)]
struct Aisle {
    from: usize,
    to: usize,
    link_type: LinkType,
}

fn create_aisles(areas: & Vec<Area>) -> Vec<Point> {
    return Vec::new();
}


fn main() {

    //let height = 100;
    //let width = 200;
    let height = 50;
    let width = 100;

    let mut areas = Vec::new();

    let mut area = Area::new();
    area.h = height;
    area.w = width;

    areas.push(area);

    generate_rooms(&mut areas);

    let aisles = create_aisles(&areas);

    let mut output = Vec::new();

    for _i in 0..height {
        let mut row = Vec::new();
        for _j in 0..width {
            row.push('*');
        }
        output.push(row);
    }

    for a in areas {
        for iy in 0..a.room.h {
            for ix in 0..a.room.w {
                let x = a.room.x + ix;
                let y = a.room.y + iy;
                output[y][x] = format!("{}", a.idx).chars().next().unwrap();
            }
        }
    }

    for p in aisles {
        output[p.y][p.x] = '-';
    }

    for i in 0..height {
        for j in 0..width {
            print!("{}", output[i][j]);
        }
        println!();
    }

}
