use std::{io::{stdout, Write, Stdout}, usize};
use crossterm::{cursor::{MoveTo, MoveLeft}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, queue, style::Print, execute, event::{self, Event, KeyCode}, style::{Attribute, SetAttribute, SetForegroundColor, Color, ResetColor, SetBackgroundColor}};
use rand::Rng;

#[allow(unused_macros)]
macro_rules! dprintln {
    ($stdout:expr, $msg:expr) => {
        execute!($stdout, LeaveAlternateScreen).unwrap();
        println!("{}", $msg);
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
               Print("@"),
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


/*
// might use Rc if multiple things with same desc could be common
enum Tile {
    Wall,
    Floor,
    Player,
    Door,
    // Enemy {desc: Box<str>},
    // Item {desc: Box<str>}
}
// */

#[derive(Debug)]
struct Room {
    pos: Rect,
    // contents: Vec<(u8, u8, Tile)>
}

#[derive(Debug, Clone, Copy)]
struct Hallway<'a> {
    entr: (Point, Point),
    rooms: (&'a Room, &'a Room)
}

enum Move {
    None,
    R,
    U,
    D,
    L,
    // ...
}

#[derive(Debug)]
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

// can be made better
fn queue_rect(stdout: &mut Stdout, rect: &Rect) {
    for y in 0..rect.h{
        for x in 0..rect.w {
            if x == 0 || x == rect.w-1 || y == 0 || y == rect.h-1 {
                queue!(stdout,
                       MoveTo(rect.x+x, rect.y+y),
                       Print("#")
                       ).unwrap();
            }
        }
    }
}

const ROOMS_X: u16 = 4;
const ROOMS_Y: u16 = 3;
const MIN_ROOM_COUNT: u32 = 8;
const HALLWAYS_SIZE: usize = ((ROOMS_X-1)*ROOMS_Y + ROOMS_X*(ROOMS_Y-1)) as usize;

fn main() -> Result<(), ()>{
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();
    let (width, height) = terminal::size().unwrap();

    if width < 80 {
        exit!(stdout, "Width of terminal window should be at least 80 characters");
    }
    if height < 30 {
        exit!(stdout, "Height of terminal window should be at least 80 characters");
    }

    let empty_room = Room {
        pos: Rect {x:0, y:0, w:0, h:0}
    };

    let mut grid: [[Option<Room>;ROOMS_X as usize];ROOMS_Y as usize] = Default::default();
    let mut room_count = 0;
    while room_count < MIN_ROOM_COUNT {
        grid = Default::default();
        room_count = 0;

        // // make them fit in the right "square" or generate them differnently
        for (r, row) in (&mut grid).iter_mut().enumerate() {
            for (c, column) in row.iter_mut().enumerate() {
                if rand::random() {
                    let r = r as u16;
                    let c = c as u16;
                    let x = rand::thread_rng().gen_range(c*width/ROOMS_X..(c+1)*width/ROOMS_X);
                    let y = rand::thread_rng().gen_range(r*height/ROOMS_Y..(r+1)*height/ROOMS_Y);

                    if (c+1)*width/ROOMS_X-x < 15 {continue};
                    if (r+1)*height/ROOMS_Y-y < 10 {continue};
                    let w = rand::thread_rng().gen_range(12..(c+1)*width/ROOMS_X-x);
                    let h = rand::thread_rng().gen_range(8..(r+1)*height/ROOMS_Y-y);

                    *column = Some(Room{pos: Rect {
                        x, y,
                        w, h,
                    }});
                    room_count += 1;
                }
            }
        }
    }

    for row in &grid {
        for column in row {
            if let Some(r) = column {
                queue_rect(&mut stdout, &r.pos);
            }
        }
    }

    let mut hallways_present =  [false; HALLWAYS_SIZE];
    let mut hallways = [Hallway{
        entr: (Point {x:0, y:0}, Point {x:0, y:0}),
        rooms: (&empty_room, &empty_room)
    };HALLWAYS_SIZE];

    let mut doors_count = 0;
    for y in 0..ROOMS_Y {
        for x in 0..ROOMS_X {
            if let Some(room) = &grid[y as usize][x as usize] {
                for x2 in x+1..ROOMS_X {
                    if let Some(room2) = &grid[y as usize][x2 as usize] {
                        let posx1 = room.pos.x + room.pos.w-1;
                        let posy1 = room.pos.y + room.pos.h/2;
                        // let posx2;
                        let posx2 = room2.pos.x;
                        let posy2 = room2.pos.y + room2.pos.h/2;
                        // offset for 2 digit numbers
                        // if doors_count < 9  {posx2 = room2.pos.x;}
                        // else                {posx2 = room2.pos.x-1;}

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

                for y2 in y+1..ROOMS_Y {
                    if let Some(room2) = &grid[y2 as usize][x as usize] {
                        let posx1 = room.pos.x + room.pos.w/2;
                        let posy1 = room.pos.y + room.pos.h-1;
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
    
    let mut position = Point{x:0, y:0};
    let mut p_move = Move::None;
    let mut curr_room;

    {
        let mut curr_room_opt = None;
        'pos: for y in 0..ROOMS_Y {
            for x in 0..ROOMS_X {
                if let Some(room) = &grid[y as usize][x as usize] {
                    curr_room_opt = Some(room);
                    position = Point {
                        x: room.pos.x + 1,
                        y: room.pos.y + 1
                    }; break 'pos;
                }
            }
        }

        if let Some(room) = curr_room_opt {curr_room = room;}
        else {unreachable!("How did it not find anycthing???")}
    }

    // TODO: might switch up and down
    // TODO: simplify hallways
    'l: loop {
        match p_move {
            Move::None => {}
            Move::R => {
                let next_pos = Point{x: position.x+1, y: position.y};
                if next_pos.x < curr_room.pos.x + curr_room.pos.w-1 {
                    queue_position_cleanup!(stdout, position);
                    position.x += 1;
                }else {
                    'h: for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break 'h}
                        else {
                            if next_pos == hallways[i].entr.0 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.1;
                                position.x += 1;
                                curr_room = hallways[i].rooms.1;
                                break 'h
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
                    'h: for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break 'h}
                        else {
                            if next_pos == hallways[i].entr.1 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.0;
                                position.x -= 1;
                                curr_room = hallways[i].rooms.0;
                                break 'h
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
                    'h: for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break 'h}
                        else {
                            if next_pos == hallways[i].entr.1 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.0;
                                position.y -= 1;
                                curr_room = hallways[i].rooms.0;
                                break 'h
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
                    'h: for i in 0..HALLWAYS_SIZE {
                        if !hallways_present[i] {break 'h}
                        else {
                            if next_pos == hallways[i].entr.0 {
                                queue_position_cleanup!(stdout, position);
                                position = hallways[i].entr.1;
                                position.y += 1;
                                curr_room = hallways[i].rooms.1;
                                break 'h
                            }
                        }
                    }
                }}
        }
        queue_position!(stdout, position);
        stdout.flush().unwrap();

        // get event for next move
        p_move = Move::None;
        if let Ok(e) = event::read() {
            match e {
                Event::Key(k) => {
                    match k.code {
                        KeyCode::Char('q') => {
                            break 'l;
                        }
                        KeyCode::Char('h') => {p_move = Move::L}
                        KeyCode::Char('j') => {p_move = Move::D}
                        KeyCode::Char('k') => {p_move = Move::U}
                        KeyCode::Char('l') => {p_move = Move::R}
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
