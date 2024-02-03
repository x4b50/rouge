use std::io::{stdout, Write, Stdout};
use crossterm::{cursor::{MoveTo, MoveLeft}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, queue, style::Print, execute, event::{self, Event, KeyCode}};

macro_rules! exit {
    ($stdout:expr, $msg:expr) => {
        execute!($stdout, LeaveAlternateScreen).unwrap();
        eprintln!($msg);
        return;
    };
}

/*
// might use Rc if multiple things with same desc could be common
enum Tile {
    Wall,
    Floor,
    Player,
    Door,
    Enemy {desc: Box<str>},
    Item {desc: Box<str>}
}

struct Room {
    width: u8,
    length: u8,
    position: (u8, u8),
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

fn main() {
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

    // let mut grid: [[Option<Room>;3];3];
    
    let rect = Rect{
        x: width/4,
        y: height/4,
        w: width/2,
        h: height/2,
    };

    queue_rect(&mut stdout, &rect);
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
}
