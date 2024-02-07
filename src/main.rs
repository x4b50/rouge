use core::panic;
use std::{io::{stdout, Stdout, Write}, usize};
use crossterm::{cursor::{MoveTo, MoveLeft}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, queue, style::Print, execute, event::{self, Event, KeyCode}, style::{Attribute, SetAttribute, SetForegroundColor, Color, ResetColor, SetBackgroundColor}};
use rand::{Rng, random};

// TODO: organise this code

// TODO: work on that to not fuck up the display
#[allow(unused_macros)]
macro_rules! dprintln {
    ($stdout:expr, $( $msg:expr ),*) => {
        execute!($stdout, LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
        println!($( $msg, )*);
        terminal::enable_raw_mode().unwrap();
        execute!($stdout, MoveLeft(u16::MAX)).unwrap();
        execute!($stdout, EnterAlternateScreen).unwrap();
    };
}

macro_rules! exit {
    ($stdout:expr, $msg:expr) => {
        execute!($stdout, LeaveAlternateScreen).unwrap();
        eprintln!($msg);
        return Err(());
    };
}

macro_rules! add_hallway {
    ($stdout:expr, $hs_pr:expr, $hs:expr, $r1:expr, $r2:expr, $count:expr, $x1:expr, $y1:expr, $x2:expr, $y2:expr) => {
        $hs_pr[$count-1] = true;
        $hs[$count-1] = Hallway{
            entr: (Point {x: $x1, y: $y1}, Point {x: $x2, y: $y2}),
            rooms: ($r1, $r2)
        };

        queue!($stdout,
               SetAttribute(Attribute::Bold),
               SetBackgroundColor(Color::Cyan),
               SetForegroundColor(Color::Black),
               MoveTo($x1, $y1), Print(format!("{}", $count)),
               if $count > 9 {
                   MoveTo($x2-1, $y2)
               } else {
                   MoveTo($x2, $y2)
               }, Print(format!("{}", $count)),
               SetAttribute(Attribute::Reset),
               ResetColor).unwrap();
    };
}

macro_rules! queue_position {
    ($stdout:expr, $position:expr) => {
        queue!($stdout,
               MoveTo($position.x, $position.y),
               Print(CHAR_PLAYER),
               MoveTo($position.x, $position.y)
              ).unwrap();
    };
}

macro_rules! queue_position_cleanup {
    ($stdout:expr, $position:expr) => {
        queue!($stdout,
               MoveTo($position.x, $position.y),
               Print(" "),
              ).unwrap();
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Room<'a> {
    pos: Rect,
    contents: &'a [Object]
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Object {
    x: u16,
    y: u16,
    content: Content
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Content {
    Item(Item),
    Enemy(Enemy)
}

#[derive(Debug, Clone, Copy)]
struct Hallway<'a> {
    entr: (Point, Point),
    rooms: (&'a Room<'a>, &'a Room<'a>)
}

enum Move {
    NONE,
    R,
    U,
    D,
    L,
    // ...
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stat {
    NONE,
    HP,
    DEF,
    ATK,
    GOLD,
    EXP,
    __Count,
}

// TODO: might get reed of kind/effect and leave 1
#[derive(Debug, Clone, Copy, PartialEq)]
struct Item {
    hidden: bool,
    kind: ItemKind,
    effect: Stat,
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ItemKind {
    NONE,
    Healing,
    Armor,
    Weapon,
    Gold,
    EXP,
    __Count,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Enemy {
    hidden: bool,
    kind: EnemyKind,
    hp: i32,
    def: i32,
    atk: i32,
    loot: Option<Item>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum EnemyKind {
    NONE,
    Ghost,
    Ogre,
    Skeleton,
    Zombie,
    __Count,
}

struct Player {
    hp: i32,
    def: i32,
    atk: i32,
    gold: i32,
    exp: i32,
    lvl: i32,
}

// can be made better
fn queue_rect(stdout: &mut Stdout, rect: &Rect) {
    for y in 0..rect.h{
        for x in 0..rect.w {
            if x == 0 || x == rect.w-1 || y == 0 || y == rect.h-1 {
                queue!(stdout,
                       MoveTo(rect.x+x, rect.y+y),
                       Print(CHAR_WALL)
                       ).unwrap();
            }
        }
    }
}

fn queue_room(stdout: &mut Stdout, room: &Room) {
    queue_rect(stdout, &room.pos);
    for obj in room.contents {
        queue!(stdout, MoveTo(room.pos.x+obj.x, room.pos.y+obj.y)).unwrap();
        match obj.content {
            Content::Item(item) => {
                if item.hidden { queue!(stdout, Print(CHAR_HIDDEN)).unwrap() }
                else {
                    match item.kind {
                        ItemKind::NONE => panic!("I don't think this should ever happen"),
                        ItemKind::__Count => panic!("This should never happen"),
                        _ => queue!(stdout, Print(CHAR_ITEM)).unwrap(),
                    }
                }
            }
            Content::Enemy(enemy) => {
                if enemy.hidden { queue!(stdout, Print(CHAR_HIDDEN)).unwrap() }
                else {
                    match enemy.kind {
                        EnemyKind::NONE => panic!("I don't think this should ever happen"),
                        EnemyKind::__Count => panic!("This should never happen"),
                        EnemyKind::Ghost => queue!(stdout, Print(CHAR_ENEMY_GHOST)).unwrap(),
                        EnemyKind::Ogre => queue!(stdout, Print(CHAR_ENEMY_OGRE)).unwrap(),
                        EnemyKind::Skeleton => queue!(stdout, Print(CHAR_ENEMY_SKELETON)).unwrap(),
                        EnemyKind::Zombie => queue!(stdout, Print(CHAR_ENEMY_ZOMBIE)).unwrap(),
                    }
                }
            }
        }
    }
}

// TODO: different level items
impl Item {
    fn random() -> Item {
        let kind = unsafe {
            std::mem::transmute::<u8, ItemKind>
                (rand::thread_rng().gen_range(1..ItemKind::__Count as u8))
        };
        Item {
            hidden: random(),
            kind,
            effect: match kind {
                ItemKind::Healing => Stat::HP,
                ItemKind::Armor => Stat::DEF,
                ItemKind::Weapon => Stat::ATK,
                ItemKind::Gold => Stat::GOLD,
                ItemKind::EXP => Stat::EXP,
                ItemKind::__Count => panic!("`Item::random()` should not be able to generate kind 'ItemKind::__Count`'"),
                ItemKind::NONE => panic!("`Item::random()` should not be able to generate kind 'ItemKind::NONE`'"),
            },
            value: rand::thread_rng().gen_range(1..10)
        }
    }
}

impl Default for Item {
    fn default() -> Self {
        Item {
            hidden: false,
            kind: ItemKind::NONE,
            effect: Stat::NONE,
            value: 0,
        }
    }
}

impl Default for Room<'static> {
    fn default() -> Self {
        Room {
            pos: Rect {
                x: 0, y: 0,
                w: 0, h: 0,
            },
            contents: &[],
        }
    }
}

// TODO: gen different stats based on type, balance a bit
impl Enemy {
    fn random() -> Enemy {
        Enemy {
            hidden: random(),
            kind: unsafe {
                std::mem::transmute
                    (rand::thread_rng().gen_range(1..EnemyKind::__Count as u8))
            },
            hp: rand::thread_rng().gen_range(3..10),
            def: rand::thread_rng().gen_range(3..10),
            atk: rand::thread_rng().gen_range(3..10),
            loot: if random() && random() {
                Some(Item::random())
            } else {None}
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy {
            hidden: false,
            kind: EnemyKind::NONE,
            hp: 0,
            def: 0,
            atk: 0,
            loot: None,
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Object {
            x: 0,
            y: 0,
            content: Content::Item(Item::default())
        }
    }
}

impl Object {
    const fn const_default() -> Self {
        Object {
            x: 0,
            y: 0,
            // TODO: keep same as Item::default()
            content: Content::Item(Item {
                    hidden: false,
                    kind: ItemKind::NONE,
                    effect: Stat::NONE,
                    value: 0,
                })
        }
    }
}

const ROOMS_X: u16 = 4;
const ROOMS_Y: u16 = 3;
const MIN_ROOM_COUNT: u32 = 8;
const HALLWAYS_SIZE: usize = ((ROOMS_X-1)*ROOMS_Y + ROOMS_X*(ROOMS_Y-1)) as usize;
const EMPTY_OBJECT: Object = Object::const_default();
const EMPTY_ROOM: Room = Room {
    pos: Rect {x:0, y:0, w:0, h:0},
    contents: &[],
};

const CHAR_WALL: char = '#';
const CHAR_PLAYER: char = '@';
const CHAR_HIDDEN: char = '?';
const CHAR_ITEM: char = 'I';
const CHAR_ENEMY_ZOMBIE: char = 'Z';
const CHAR_ENEMY_SKELETON: char = 'S';
const CHAR_ENEMY_GHOST: char = 'G';
const CHAR_ENEMY_OGRE: char = 'O';

// TODO: make a shop mechanic or sth to make gold usefull
const ITEMS_MAX: u32 = MIN_ROOM_COUNT*3/4;
const ITEMS_MIN: u32 = ITEMS_MAX/2;
const MONSTERS_MAX: u32 = MIN_ROOM_COUNT;
const MONSTERS_MIN: u32 = MONSTERS_MAX/2;

// const ITEM_NAMES: [&str; 4] = [
    // "Armor",
    // "Sword",
    // "Healing kit",
    // "Pile of gold",
// ];

fn main() -> Result<(), ()>{
    let mut stdout = stdout();
    let (width, height) = terminal::size().unwrap();

    if width < 80 {exit!(stdout, "Width of terminal window should be at least 80 characters");}
    if height < 30 {exit!(stdout, "Height of terminal window should be at least 80 characters");}
    let height = height-2;

    let mut grid = [[None;ROOMS_X as usize];ROOMS_Y as usize];
    let mut room_count = 0;
    while room_count < MIN_ROOM_COUNT {
        for (r, row) in (&mut grid).iter_mut().enumerate() {
            for (c, column) in row.iter_mut().enumerate() {
                if column.is_none() && random() {
                    let r = r as u16;
                    let c = c as u16;
                    let x = rand::thread_rng().gen_range(c*width/ROOMS_X+1..(c+1)*width/ROOMS_X);
                    let y = rand::thread_rng().gen_range(r*height/ROOMS_Y+1..(r+1)*height/ROOMS_Y);

                    if (c+1)*width/ROOMS_X-x < 15 {continue};
                    if (r+1)*height/ROOMS_Y-y < 10 {continue};
                    let w = rand::thread_rng().gen_range(12..(c+1)*width/ROOMS_X-x);
                    let h = rand::thread_rng().gen_range(8..(r+1)*height/ROOMS_Y-y);

                    *column = Some(Room{
                        pos: Rect {
                            x, y,
                            w, h,
                        },
                        contents: &[EMPTY_OBJECT]
                    });
                    room_count += 1;
                }
            }
        }
    }
    
    let mut position = Point{x:0, y:0};
    let mut moved = Move::NONE;
    let mut c_room = Point {x: u16::MAX, y: u16::MAX};

    'pos: for y in 0..ROOMS_Y {
        for x in 0..ROOMS_X {
            if let Some(room) = grid[y as usize][x as usize] {
                c_room = Point { x, y };
                position = Point {
                    x: room.pos.x + 1,
                    y: room.pos.y + 1
                }; break 'pos;
            }
        }
    }

    // let contents = vec![Object::default();room_count as usize].into_boxed_slice();
    let mut contents: [[Vec<Object>; ROOMS_X as usize]; ROOMS_Y as usize] = Default::default();

    {
        let items_cout = rand::thread_rng().gen_range(ITEMS_MIN..=ITEMS_MAX);
        for _ in 0..items_cout {
            loop {
                let y = rand::thread_rng().gen_range(0..ROOMS_Y) as usize;
                let x = rand::thread_rng().gen_range(0..ROOMS_X) as usize;
                if let Some(room) = grid[y][x] {
                    contents[y][x].push(Object {
                        x: rand::thread_rng().gen_range(1..room.pos.w-1),
                        y: rand::thread_rng().gen_range(1..room.pos.h-1),
                        content: Content::Item(Item::random())
                    });
                    break;
                }
            }
        }

        let enemies_count = rand::thread_rng().gen_range(MONSTERS_MIN..=MONSTERS_MAX);
        for _ in 0..enemies_count {
            loop {
                let y = rand::thread_rng().gen_range(0..ROOMS_Y) as usize;
                let x = rand::thread_rng().gen_range(0..ROOMS_X) as usize;
                if let Some(room) = grid[y][x] {
                    if c_room.y != y as u16 && c_room.x != x as u16 {
                        contents[y][x].push(Object {
                            x: rand::thread_rng().gen_range(1..room.pos.w-1),
                            y: rand::thread_rng().gen_range(1..room.pos.h-1),
                            content: Content::Enemy(Enemy::random())
                        });
                        break;
                    }
                }
            }
        }
    }

    for y in 0..ROOMS_Y as usize {
        for x in 0..ROOMS_X as usize {
            if let Some(room) = &mut grid[y][x] {room.contents = &contents[y][x];}
        }
    }

    let mut curr_room;
    if let Some(room) = &grid[c_room.y as usize][c_room.x as usize] {
        curr_room = room;
    } else {unreachable!("there should be a staring room")}

    // switch to the game screen
    execute!(stdout, EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();

    for row in &grid {
        for column in row {
            if let Some(room) = column {
                queue_room(&mut stdout, room);
            }
        }
    }

    let mut hallways_present =  [false; HALLWAYS_SIZE];
    let mut hallways = [Hallway{
        entr: (Point {x:0, y:0}, Point {x:0, y:0}),
        rooms: (&EMPTY_ROOM, &EMPTY_ROOM)
    };HALLWAYS_SIZE];

    let mut doors_count = 0;
    for y in 0..ROOMS_Y as usize {
        for x in 0..ROOMS_X as usize {
            if let Some(room) = &grid[y][x] {
                let posx1 = room.pos.x + room.pos.w-1;
                let posy1 = room.pos.y + room.pos.h/2;
                for x2 in x+1..ROOMS_X as usize {
                    if let Some(room2) = &grid[y][x2] {
                        let posx2 = room2.pos.x;
                        let posy2 = room2.pos.y + room2.pos.h/2;

                        doors_count += 1;
                        add_hallway!(stdout,
                                     hallways_present, hallways,
                                     room, room2,
                                     doors_count,
                                     posx1, posy1,
                                     posx2, posy2);
                        break;
                    }
                }

                let posx1 = room.pos.x + room.pos.w/2;
                let posy1 = room.pos.y + room.pos.h-1;
                for y2 in y+1..ROOMS_Y as usize {
                    if let Some(room2) = &grid[y2][x] {
                        let posx2 = room2.pos.x + room2.pos.w/2;
                        let posy2 = room2.pos.y;

                        doors_count += 1;
                        add_hallway!(stdout,
                                     hallways_present, hallways,
                                     room, room2,
                                     doors_count,
                                     posx1, posy1,
                                     posx2, posy2);
                        break;
                    }
                }
            }
        }
    }
    stdout.flush().unwrap();

    // TODO: might switch up and down
    // TODO: simplify hallways
    'l: loop {
        match moved {
            Move::NONE => {}
            Move::R => {
                let next_pos = Point{x: position.x+1, y: position.y};
                if next_pos.x < curr_room.pos.x + curr_room.pos.w-1 {
                    queue_position_cleanup!(stdout, position);
                    position.x += 1;
                }else {
                    for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break}
                        else {
                            if next_pos == hallways[i].entr.0 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.1;
                                position.x += 1;
                                curr_room = hallways[i].rooms.1;
                                break
                            }
                        }
                    }
                }}
            Move::L => {
                let next_pos = Point{x: position.x-1, y: position.y};
                if next_pos.x > curr_room.pos.x {
                    queue_position_cleanup!(stdout, position);
                    position.x -= 1;
                }else {
                    for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break}
                        else {
                            if next_pos == hallways[i].entr.1 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.0;
                                position.x -= 1;
                                curr_room = hallways[i].rooms.0;
                                break
                            }
                        }
                    }
                }}
            Move::U => {
                let next_pos = Point{x: position.x, y: position.y-1};
                if next_pos.y > curr_room.pos.y {
                    queue_position_cleanup!(stdout, position);
                    position.y -= 1;
                }else {
                    for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break}
                        else {
                            if next_pos == hallways[i].entr.1 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.0;
                                position.y -= 1;
                                curr_room = hallways[i].rooms.0;
                                break
                            }
                        }
                    }
                }}
            Move::D => {
                let next_pos = Point{x: position.x, y: position.y+1};
                if next_pos.y < curr_room.pos.y + curr_room.pos.h-1 {
                    queue_position_cleanup!(stdout, position);
                    position.y += 1;
                }else {
                    for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break}
                        else {
                            if next_pos == hallways[i].entr.0 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.1;
                                position.y += 1;
                                curr_room = hallways[i].rooms.1;
                                break
                            }
                        }
                    }
                }}
        }
        queue_position!(stdout, position);
        stdout.flush().unwrap();

        // get event for next move
        moved = Move::NONE;
        if let Ok(e) = event::read() {
            match e {
                Event::Key(k) => {
                    match k.code {
                        KeyCode::Char('q') => {break 'l;}
                        KeyCode::Char('h') => {moved = Move::L}
                        KeyCode::Char('j') => {moved = Move::D}
                        KeyCode::Char('k') => {moved = Move::U}
                        KeyCode::Char('l') => {moved = Move::R}
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
    Ok(())
}
