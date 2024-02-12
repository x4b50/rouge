use rand::{Rng, random};
use core::panic;
use std::io::Stdout;
use crossterm::{cursor::{MoveTo, MoveRight}, queue, style::Print};

#[derive(Debug, PartialEq, Clone)]
pub struct Room {
    pub pos: Rect,
    pub contents: Vec<Object>
}

#[derive(Debug, Clone, Copy)]
pub struct Hallway {
    pub entr: [Point;2],
    pub rooms: [Index;2]
}

// TODO: idk if keeping an index instead for ex. compying rect is a good idea
#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub room: Rect
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Index {
    pub x: usize,
    pub y: usize
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Object {
    pub hidden: bool,
    pub x: u16,
    pub y: u16,
    pub content: Content
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Content {
    Item(Item),
    Enemy(Enemy),
    Entrance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    NONE,
    HP,
    DEF,
    ATK,
    GOLD,
    EXP,
    __Count,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub effect: Stat,
    pub value: i16,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: i16,
    pub def: i16,
    pub atk: i16,
    pub loot: Item,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EnemyKind {
    NONE,
    Goblin,
    Ogre,
    Skeleton,
    Zombie,
    __Count,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16
}

gen_menu!(struct Player {
    lvl: i16,
    exp: i16,
    hp: i16,
    def: i16,
    atk: i16,
    gold: i16,
});

pub struct Combat {
    pub blocks: i16,
    pub buffs: i16,
    pub dodge: bool,
    pub action: CMove,
}

#[derive(PartialEq)]
pub enum Move {
    NONE,
    R,
    L,
    D,
    U,
}

#[derive(PartialEq)]
pub enum CMove {
    NONE,
    Attack,
    Block,
    Dodge,
    Buff,
    Run,
}

impl Player {
    pub fn random() -> Player {
        Player {
            hp: rand::thread_rng().gen_range(15..=25),
            def: rand::thread_rng().gen_range(5..13),
            atk: rand::thread_rng().gen_range(5..13),
            gold: 0,
            exp: 0,
            lvl: 1,
        }
    }
}

// TODO: different level items
impl Item {
    pub fn random() -> Item {
        Item {
            effect: unsafe {
                std::mem::transmute::<u8, Stat>
                    (rand::thread_rng().gen_range(1..Stat::__Count as u8))
            },
            value: rand::thread_rng().gen_range(2..7)
        }
    }
}

// TODO: gen different stats based on type, balance a bit
impl Enemy {
    pub fn random(mult: i16) -> Enemy {
        Enemy {
            kind: unsafe {
                std::mem::transmute
                    (rand::thread_rng().gen_range(1..EnemyKind::__Count as u8))
            },
            hp: rand::thread_rng().gen_range(7*mult..20*mult),
            def: rand::thread_rng().gen_range(7*mult-4..10*mult),
            atk: rand::thread_rng().gen_range(7*mult-4..10*mult),
            loot: if random() && random() {
                Item::random()
            } else {
                let mut item = Item::random();
                item.effect = Stat::EXP;
                item
            }
        }
    }
}

impl EnemyKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            EnemyKind::NONE => panic!("`to_str` should never be called on NONE"),
            EnemyKind::__Count => panic!("`to_str` should never be called on __Count"),
            EnemyKind::Goblin => &"Goblin",
            EnemyKind::Ogre => &"Ogre",
            EnemyKind::Skeleton => &"Skeleton",
            EnemyKind::Zombie => &"Zombie",
        }
    }
}

impl Combat {
    pub fn new() -> Combat {
        Combat {
            blocks: 0,
            buffs: 0,
            dodge: false,
            action: CMove::NONE,
        }
    }
}

pub mod macros {
    #[macro_export]
    #[allow(unused_macros)]
    macro_rules! dprintln {
        ($( $msg:expr ),*) => {
            use crossterm::cursor::MoveLeft;
            std::io::stdout().flush().unwrap();
            execute!(std::io::stderr(), LeaveAlternateScreen).unwrap();
            terminal::disable_raw_mode().unwrap();
            println!($( $msg, )*);
            terminal::enable_raw_mode().unwrap();
            execute!(std::io::stderr(), MoveLeft(u16::MAX)).unwrap();
            execute!(std::io::stderr(), EnterAlternateScreen).unwrap();
        };
    }

    #[macro_export]
    macro_rules! exit {
        ($stdout:expr, $msg:expr) => {
            execute!($stdout, LeaveAlternateScreen).unwrap();
            eprintln!($msg);
            return Err(());
        };
    }

    #[macro_export]
    macro_rules! queue_position {
        ($stdout:expr, $position:expr) => {
            queue!($stdout,
                   MoveTo($position.x, $position.y),
                   Print(CHAR_PLAYER),
                   MoveTo($position.x, $position.y),
                  ).unwrap();
        };
    }

    #[macro_export]
    macro_rules! queue_enemy_cleanup {
        ($stdout:expr, $position:expr, $obj:expr) => {
            queue!($stdout,
                   MoveTo($position.room.x + $obj.x, $position.room.y + $obj.y),
                   Print(CHAR_FLOOR),
                   ).unwrap();
        };
    }

    #[macro_export]
    macro_rules! queue_position_cleanup {
        ($stdout:expr, $position:expr) => {
            queue!($stdout,
                   MoveTo($position.x, $position.y),
                   Print(CHAR_FLOOR),
                   ).unwrap();
        };
    }

    #[macro_export]
    macro_rules! check_move {
        ($stdout:expr, $pos:expr, $grid:expr, $hs_p:expr, $hs:expr, $axis:tt, $sign:tt) => {
            let next_pos = match stringify!($axis) {
                "x" => Point{x: $pos.x $sign 1, y: $pos.y},
                "y" => Point{x: $pos.x, y: $pos.y $sign 1},
                _ => panic!("invalid axis, should be `x` or `y`")
                    // _ => compile_error!("only accepts `x` and `y`")
            };

            let cond = match (stringify!($axis), stringify!($sign)) {
                ("x", "+") => next_pos.x < $pos.room.x + $pos.room.w-1,
                ("x", "-") => next_pos.x > $pos.room.x,
                ("y", "+") => next_pos.y < $pos.room.y + $pos.room.h-1,
                ("y", "-") => next_pos.y > $pos.room.y,
                _ => panic!("direction signifier should be either `+` or `-`")
            };

            if cond {
                queue_position_cleanup!($stdout, $pos);
                $pos.$axis = $pos.$axis $sign 1;
            } else {
                let hw_idx = (0.5 $sign 0.5) as usize;
                for i in 0..HALLWAYS_SIZE {
                    if !$hs_p[i] {break}
                    else {
                        if next_pos == $hs[i].entr[1-hw_idx] {
                            queue_position_cleanup!($stdout, $pos);
                            $pos.x = $hs[i].entr[hw_idx].x;
                            $pos.y = $hs[i].entr[hw_idx].y;
                            $pos.$axis = $pos.$axis $sign 1;
                            let idx = $hs[i].rooms[hw_idx];
                            if let Some(room) = &$grid[idx.y][idx.x] {
                                $pos.room = room.pos.clone();
                            } else {unreachable!("hallways should hold indexes of valid rooms")}
                            break
                        }
                    }
                }
            }
        };
    }


    #[macro_export]
    macro_rules! replace_expr {
        ($_t:tt $sub:expr) => {$sub};
    }

    // magic
    // https://danielkeep.github.io/tlborm/book/blk-counting.html
    // https://stackoverflow.com/questions/34304593/counting-length-of-repetition-in-macro
    // https://stackoverflow.com/questions/37140768/how-to-get-struct-field-names-in-rust
    #[macro_export]
    macro_rules! gen_menu {
        (struct $name:ident {$($field:ident: $type:ty),+ $(,)* }) => {
            pub struct $name {
                $(pub $field: $type),*
            }

            const MENU_WIDTH: u16 = 0 $( +stringify!($field).len() as u16 +4)*;
            const N_PADDS: u16 = {<[()]>::len(&[$(replace_expr!($field ())),*])} as u16 + 1;
            pub fn queue_menu(stdout: &mut Stdout, player: &$name, width: u16, height: u16) {
                let padding = (width-MENU_WIDTH)/N_PADDS;
                queue!(stdout,
                       MoveTo(0, height),
                       Print("-".repeat(width.into())),
                       MoveTo(0, height+1),
                       Print(" ".repeat(width.into())),
                       MoveTo(0, height+1),
                       $(
                           MoveRight(padding),
                           Print(stringify!($field)),
                           Print(": "),
                           Print(player.$field),
                        )*
                      ).unwrap();
            }
        };
    }
}
