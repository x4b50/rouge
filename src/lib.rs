use core::panic;
use rand::{Rng, random};

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

pub mod macros {
    #[macro_export]
    macro_rules! exit {
        ($stdout:expr, $msg:expr) => {
            execute!($stdout, LeaveAlternateScreen).unwrap();
            eprintln!($msg);
            return Err(());
        };
    }

    #[macro_export]
    macro_rules! add_hallway {
        ($stdout:expr, $hs_pr:expr, $hs:expr, $r1:expr, $r2:expr, $count:expr, $x1:expr, $y1:expr, $x2:expr, $y2:expr) => {
            // why tho
            use crossterm::{style::{Attribute, SetAttribute, SetForegroundColor, Color, ResetColor, SetBackgroundColor}};
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

    #[macro_export]
    macro_rules! queue_position {
        ($stdout:expr, $position:expr) => {
            queue!($stdout,
                   MoveTo($position.x, $position.y),
                   Print(CHAR_PLAYER),
                   MoveTo($position.x, $position.y)
                  ).unwrap();
        };
    }

    #[macro_export]
    macro_rules! queue_position_cleanup {
        ($stdout:expr, $position:expr) => {
            queue!($stdout,
                   MoveTo($position.x, $position.y),
                   Print(" "),
                   ).unwrap();
        };
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Room<'a> {
    pub pos: Rect,
    pub contents: &'a [Object]
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Object {
    pub x: u16,
    pub y: u16,
    pub content: Content
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Content {
    Item(Item),
    Enemy(Enemy)
}

#[derive(Debug, Clone, Copy)]
pub struct Hallway<'a> {
    pub entr: (Point, Point),
    pub rooms: (&'a Room<'a>, &'a Room<'a>)
}

pub enum Move {
    NONE,
    R,
    U,
    D,
    L,
    // ...
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16
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

// TODO: might get reed of kind/effect and leave 1
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub hidden: bool,
    pub kind: ItemKind,
    pub effect: Stat,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemKind {
    NONE,
    Healing,
    Armor,
    Weapon,
    Gold,
    EXP,
    __Count,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Enemy {
    pub hidden: bool,
    pub kind: EnemyKind,
    pub hp: i32,
    pub def: i32,
    pub atk: i32,
    pub loot: Option<Item>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EnemyKind {
    NONE,
    Ghost,
    Ogre,
    Skeleton,
    Zombie,
    __Count,
}

struct Player {
    pub hp: i32,
    pub def: i32,
    pub atk: i32,
    pub gold: i32,
    pub exp: i32,
    pub lvl: i32,
}

// TODO: different level items
impl Item {
    pub fn random() -> Item {
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

// TODO: gen different stats based on type, balance a bit
impl Enemy {
    pub fn random() -> Enemy {
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
