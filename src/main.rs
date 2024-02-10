use rouge::*;
use std::{io::{stdout, Stdout, Write}, usize};
use crossterm::{cursor::{DisableBlinking, MoveTo, MoveLeft}, event::{self, Event, KeyCode}, execute, queue, style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, SetTitle}};
use rand::{Rng, random};

const ROOMS_X: u16 = 4;
const ROOMS_Y: u16 = 3;
const MIN_ROOM_COUNT: u32 = 8;
const HALLWAYS_SIZE: usize = ((ROOMS_X-1)*ROOMS_Y + ROOMS_X*(ROOMS_Y-1)) as usize;
const EMPTY_ROOM: Room = Room {
    pos: Rect {x:0, y:0, w:0, h:0},
    contents: vec![],
};

const CHAR_WALL: char = '#';
const CHAR_FLOOR: char = ' ';
const CHAR_PLAYER: char = '@';
const CHAR_HIDDEN: char = '?';
const CHAR_ITEM: char = 'I';
const CHAR_ENEMY_ZOMBIE: char = 'Z';
const CHAR_ENEMY_SKELETON: char = 'S';
const CHAR_ENEMY_GOBLIN: char = 'G';
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

    let mut player = Player::random();
    let mut grid: [[Option<Room>;ROOMS_X as usize];ROOMS_Y as usize] = Default::default();
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
                        contents: vec![]
                    });
                    room_count += 1;
                }
            }
        }
    }

    let mut position = Position {x:0, y:0, room: Rect {x:0, y:0, w:0, h:0}};
    'p: for y in 0..ROOMS_Y as usize {
        for x in 0..ROOMS_X as usize {
            if let Some(room) = &grid[y][x] {
                position = Position {
                    x: room.pos.x+1, y:room.pos.y+1,
                    room: room.pos.clone()
                }; break 'p;
            }
        }
    }

    { // do not keep contents around
        let mut contents: [[Vec<Object>; ROOMS_X as usize]; ROOMS_Y as usize] = Default::default();
        let items_cout = rand::thread_rng().gen_range(ITEMS_MIN..=ITEMS_MAX);
        for _ in 0..items_cout {
            loop {
                let y = rand::thread_rng().gen_range(0..ROOMS_Y) as usize;
                let x = rand::thread_rng().gen_range(0..ROOMS_X) as usize;
                if let Some(room) = &grid[y][x] {
                    contents[y][x].push(Object {
                        hidden: random(),
                        removed: false,
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
                if let Some(room) = &grid[y][x] {
                    if room.pos != position.room {
                        contents[y][x].push(Object {
                            hidden: random(),
                            removed: false,
                            x: rand::thread_rng().gen_range(1..room.pos.w-1),
                            y: rand::thread_rng().gen_range(1..room.pos.h-1),
                            content: Content::Enemy(Enemy::random())
                        });
                        break;
                    }
                }
            }
        }

        for y in 0..ROOMS_Y as usize {
            for x in 0..ROOMS_X as usize {
                if let Some(room) = &mut grid[y][x] {room.contents = contents[y][x].clone();}
            }
        }
    }

    // switch to the game screen
    execute!(stdout, EnterAlternateScreen, SetTitle("Rouge"), DisableBlinking).unwrap();
    terminal::enable_raw_mode().unwrap();
    for row in &grid {
        for column in row {
            if let Some(room) = column {
                queue_rect(&mut stdout, &room.pos);
            }
        }
    }

    let mut hs_present = [false; HALLWAYS_SIZE];
    let mut hallways = [Hallway{
        entr: [Point{x:0,y:0};2],
        rooms: [Index{x:0,y:0};2],
    }; HALLWAYS_SIZE];

    let mut doors_count = 0;
    for y in 0..ROOMS_Y as usize {
        for x in 0..ROOMS_X as usize {
            if let Some(room) = &grid[y][x] {
                for x2 in x+1..ROOMS_X as usize {
                    if let Some(room2) = &grid[y][x2] {
                        add_hallway(&mut stdout, &mut hs_present, &mut hallways, &mut doors_count,
                                    x, y, x2, y, room, room2, false);
                        break;
                    }
                }

                for y2 in y+1..ROOMS_Y as usize {
                    if let Some(room2) = &grid[y2][x] {
                        add_hallway(&mut stdout, &mut hs_present, &mut hallways, &mut doors_count,
                                    x, y, x, y2, room, room2, true);
                        break;
                    }
                }
            }
        }
    }

    let mut moved = Move::NONE;
    loop {
        // rendering -----------------------------------------------------------
        for row in &grid {
            for column in row {
                if let Some(room) = column {
                    queue_room(&mut stdout, room, &position);
                }
            }
        }
        queue_menu(&mut stdout, &player, width, height);
        queue_position!(stdout, position);
        stdout.flush().unwrap();
        // rendering -----------------------------------------------------------

        // events --------------------------------------------------------------
        if let Ok(e) = event::read() {
            match e {
                Event::Key(k) => {
                    match k.code {
                        KeyCode::Char('q') => {break;}
                        KeyCode::Char('h') => {moved = Move::L}
                        KeyCode::Char('j') => {moved = Move::D}
                        KeyCode::Char('k') => {moved = Move::U}
                        KeyCode::Char('l') => {moved = Move::R}
                        _ => {moved = Move::NONE}
                    }
                }
                _ => {}
            }
        }
        // events --------------------------------------------------------------

        // logic ---------------------------------------------------------------
        match moved {
            Move::NONE => {}
            Move::R => {check_move!(stdout, position, grid, hs_present, hallways, x, +);}
            Move::L => {check_move!(stdout, position, grid, hs_present, hallways, x, -);}
            Move::D => {check_move!(stdout, position, grid, hs_present, hallways, y, +);}
            Move::U => {check_move!(stdout, position, grid, hs_present, hallways, y, -);}
        }

        let mut item_to_drop = None;
        let y = (position.y *ROOMS_Y/height) as usize;
        let x = (position.x *ROOMS_X/width) as usize;
        if let Some(room) = &grid[y][x] {
            for (o, obj) in room.contents.iter().enumerate() {
                if obj.x == position.x - position.room.x && obj.y == position.y - position.room.y && !obj.removed {
                    match obj.content {
                        Content::Item(item) => {
                            match item.effect {
                                Stat::NONE => unreachable!("`NONE` variant of `Stat` should not be accessable here"),
                                Stat::__Count => unreachable!("`__Count` variant of `Stat` should never be constructed"),
                                Stat::HP => { item_to_drop = Some(o);
                                    player.hp += item.value;
                                }
                                Stat::DEF => { item_to_drop = Some(o);
                                    player.def += item.value;
                                }
                                Stat::ATK => { item_to_drop = Some(o);
                                    player.atk += item.value;
                                }
                                Stat::GOLD => { item_to_drop = Some(o);
                                    player.gold += item.value;
                                }
                                Stat::EXP => { item_to_drop = Some(o);
                                    player.exp += item.value;
                                }
                            }
                        }
                        Content::Enemy(_) => todo!()
                    }
                    break
                }
            }
        }

        if let Some(o) = item_to_drop {
            if let Some(room) = &mut grid[y][x] {
                room.contents.remove(o);
            } else {unreachable!("How did you remove item from empty room")}
        }
        // logic ---------------------------------------------------------------
    }

    execute!(stdout, LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
    Ok(())
}

fn add_hallway(stdout: &mut Stdout, hs_present: &mut [bool], hallways: &mut [Hallway], doors_count: &mut usize,
               x1: usize, y1: usize, x2: usize, y2: usize, r1: &Room, r2: &Room, vert: bool) {
    let (ax1, ay1, ax2, ay2) = if !vert {(
        r1.pos.x + r1.pos.w-1,
        r1.pos.y + r1.pos.h/2,
        r2.pos.x,
        r2.pos.y + r2.pos.h/2,
    )} else {(
        r1.pos.x + r1.pos.w/2,
        r1.pos.y + r1.pos.h-1,
        r2.pos.x + r2.pos.w/2,
        r2.pos.y,
    )};
    hs_present[*doors_count] = true;
    hallways[*doors_count] = Hallway {
        entr: [Point {x: ax1, y: ay1}, Point {x: ax2, y: ay2}],
        rooms: [Index {x: x1, y: y1}, Index {x:x2, y:y2}]
    };
    *doors_count += 1;
    queue!(stdout,
           SetAttribute(Attribute::Bold),
           SetBackgroundColor(Color::Cyan),
           SetForegroundColor(Color::Black),
           MoveTo(ax1, ay1), Print(format!("{}", doors_count)),
           if *doors_count > 9 {
               MoveTo(ax2-1, ay1)
           } else {
               MoveTo(ax2, ay2)
           }, Print(format!("{}", doors_count)),
           SetAttribute(Attribute::Reset),
           ResetColor).unwrap();
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
            } else {
                queue!(stdout,
                       MoveTo(rect.x+x, rect.y+y),
                       Print(CHAR_FLOOR)
                       ).unwrap();
            }
        }
    }
}

fn queue_room(stdout: &mut Stdout, room: &Room, position: &Position) {
    for obj in &room.contents {
        if !obj.removed {
            queue!(stdout, MoveTo(room.pos.x+obj.x, room.pos.y+obj.y)).unwrap();
            if obj.hidden && room.pos != position.room {queue!(stdout, Print(CHAR_HIDDEN)).unwrap()}
            else {
                match obj.content {
                    Content::Item(item) => {
                        match item.effect {
                            Stat::NONE => panic!("I don't think this should ever happen"),
                            Stat::__Count => panic!("This should never happen"),
                            _ => queue!(stdout, Print(CHAR_ITEM)).unwrap(),
                        }
                    }
                    Content::Enemy(enemy) => {
                        match enemy.kind {
                            EnemyKind::NONE => panic!("I don't think this should ever happen"),
                            EnemyKind::__Count => panic!("This should never happen"),
                            EnemyKind::Goblin => queue!(stdout, Print(CHAR_ENEMY_GOBLIN)).unwrap(),
                            EnemyKind::Ogre => queue!(stdout, Print(CHAR_ENEMY_OGRE)).unwrap(),
                            EnemyKind::Skeleton => queue!(stdout, Print(CHAR_ENEMY_SKELETON)).unwrap(),
                            EnemyKind::Zombie => queue!(stdout, Print(CHAR_ENEMY_ZOMBIE)).unwrap(),
                        }
                    }
                }
            }
        }
    }
}
