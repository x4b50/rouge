use std::{io::{stdout, Write, Stdout}, usize};
use crossterm::{cursor::{MoveTo, MoveLeft}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, queue, style::Print, execute, event::{self, Event, KeyCode}, style::{Attribute, SetAttribute, SetForegroundColor, Color, ResetColor, SetBackgroundColor}};
use rand::Rng;

macro_rules! exit {
    ($stdout:expr, $msg:expr) => {
        execute!($stdout, LeaveAlternateScreen).unwrap();
        eprintln!($msg);
        return Err(());
    };
}

// /*
// might use Rc if multiple things with same desc could be common
enum Tile {
    Wall,
    Floor,
    Player,
    Door,
    // Enemy {desc: Box<str>},
    // Item {desc: Box<str>}
}

#[derive(Debug)]
struct Room {
    pos: Rect,
    // contents: Vec<(u8, u8, Tile)>
}

enum Move {
    R,
    U,
    D,
    L,
    DashR,
    DashU,
    DashD,
    DashL,
    // ...
}
// */

#[derive(Debug)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16
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

    macro_rules! add_hallway {
        ($count:expr, $move1:expr, $move2:expr) => {
            queue!(stdout,
                   SetAttribute(Attribute::Bold),
                   SetBackgroundColor(Color::Cyan),
                   SetForegroundColor(Color::Black),
                   $move1, Print(format!("{}", $count)),
                   $move2, Print(format!("{}", $count)),
                   SetAttribute(Attribute::Reset),
                   ResetColor).unwrap();
        };
    }

    let mut doors_count = 0;
    for y in 0..ROOMS_Y {
        for x in 0..ROOMS_X {
            if let Some(room) = &grid[y as usize][x as usize] {
                for x2 in x+1..ROOMS_X {
                    if let Some(room2) = &grid[y as usize][x2 as usize] {
                        doors_count += 1;
                        add_hallway!(doors_count,
                                     MoveTo(room.pos.x + room.pos.w-1, room.pos.y + room.pos.h/2),
                                     if doors_count < 10 {
                                         MoveTo(room2.pos.x, room2.pos.y + room2.pos.h/2)
                                     } else {
                                         MoveTo(room2.pos.x-1, room2.pos.y + room2.pos.h/2)
                                     });
                        break;
                    }
                }

                for y2 in y+1..ROOMS_Y {
                    if let Some(room2) = &grid[y2 as usize][x as usize] {
                        doors_count += 1;
                        add_hallway!(doors_count,
                                     MoveTo(room.pos.x + room.pos.w/2, room.pos.y + room.pos.h-1),
                                     MoveTo(room2.pos.x + room2.pos.w/2, room2.pos.y)
                                     );
                        break;
                    }
                }
            }
        }
    }
    
    stdout.flush().unwrap();
    execute!(stdout, MoveTo(1,1)).unwrap();

    'l: loop {
        if let Ok(e) = event::read() {
            match e {
                Event::Key(k) => {
                    if KeyCode::Char('q') == k.code {
                        break 'l;
                    }
                    if let KeyCode::Char(c) = k.code {
                        execute!(stdout, Print(format!("{}\n", c))).unwrap();
                        // execute!(stdout, MoveLeft(u16::MAX)).unwrap();
                        execute!(stdout, MoveLeft(1)).unwrap();
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
