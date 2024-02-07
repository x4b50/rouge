use rouge::*;
use core::panic;
use std::{io::{stdout, Stdout, Write}, usize};
use crossterm::{cursor::MoveTo, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, queue, style::Print, execute, event::{self, Event, KeyCode}};
use rand::{Rng, random};

// TODO: organise this code

const ROOMS_X: u16 = 4;
const ROOMS_Y: u16 = 3;
const MIN_ROOM_COUNT: u32 = 8;
const HALLWAYS_SIZE: usize = ((ROOMS_X-1)*ROOMS_Y + ROOMS_X*(ROOMS_Y-1)) as usize;
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

    let player = Player::random();
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
                        contents: &[]
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

    // TODO: chekc for content position collisions
    let mut contents: [[Vec<Object>; ROOMS_X as usize]; ROOMS_Y as usize] = Default::default();

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
    loop {
        // logic ---------------------------------------------------------------
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
        // logic ---------------------------------------------------------------

        // rendering -----------------------------------------------------------
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
    }

    execute!(stdout, LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
    Ok(())
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
