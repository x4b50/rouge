use rouge::*;
use std::{io::{stdout, Stdout, Write}, usize};
use crossterm::{cursor::{DisableBlinking, MoveTo}, event::{self, Event, KeyCode}, execute, queue, style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, SetTitle}};
use rand::{Rng, random};

const ROOMS_X: u16 = 4;
const ROOMS_Y: u16 = 3;
const MIN_ROOM_COUNT: u16 = 8;
const HALLWAYS_SIZE: usize = ((ROOMS_X-1)*ROOMS_Y + ROOMS_X*(ROOMS_Y-1)) as usize;

const CHAR_WALL: char = '#';
const CHAR_FLOOR: char = ' ';
const CHAR_PLAYER: char = '@';
const CHAR_HIDDEN: char = '?';
const CHAR_ITEM: char = 'I';
const CHAR_ENEMY_ZOMBIE: char = 'Z';
const CHAR_ENEMY_SKELETON: char = 'S';
const CHAR_ENEMY_GOBLIN: char = 'G';
const CHAR_ENEMY_OGRE: char = 'O';
const CHAR_ENTRANCE: char = '*';

const ITEMS_MAX: u16 = MIN_ROOM_COUNT*3/4;
const ITEMS_MIN: u16 = ITEMS_MAX/2;
const MONSTERS_MAX: u16 = MIN_ROOM_COUNT;
const MONSTERS_MIN: u16 = MONSTERS_MAX*2/3;

const MULT_ATK: i16 = 3;
const MULT_DEF: i16 = 3;

// const COMBAT;

const LEVELUP: &str = "Level up!";
const PICKUP_HP: &str = "You've picked up a healing kit";
const PICKUP_DEF: &str = "You've picked up armor";
const PICKUP_ATK: &str = "You've picked up a weapon";
const PICKUP_GOLD: &str = "You've picked up gold";
const PICKUP_EXP: &str = "You've picked up experience points";

fn main() -> Result<(), ()> {
    let mut stdout = stdout();
    let (width, height) = terminal::size().unwrap();

    if width < 80 {exit!(stdout, "Width of terminal window should be at least 80 characters");}
    if height < 42 {exit!(stdout, "Height of terminal window should be at least 42 characters");}
    let height = height-2;
    let frame = Rect {
        x: width/4,
        y: height/3,
        w: width/2,
        h: height/3,
    };

    let mut player = Player::random();
    let mut died = false;
    let mut exited = false;

    'game: loop {
        let mut grid: [[Option<Room>;ROOMS_X as usize];ROOMS_Y as usize] = Default::default();
        {
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
                            add_hallway(&mut hs_present, &mut hallways, &mut doors_count,
                                        x, y, x2, y, room, room2, false);
                            break;
                        }
                    }

                    for y2 in y+1..ROOMS_Y as usize {
                        if let Some(room2) = &grid[y2][x] {
                            add_hallway(&mut hs_present, &mut hallways, &mut doors_count,
                                        x, y, x, y2, room, room2, true);
                            break;
                        }
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
                                x: rand::thread_rng().gen_range(1..room.pos.w-1),
                                y: rand::thread_rng().gen_range(1..room.pos.h-1),
                                content: Content::Enemy(Enemy::random(player.lvl))
                            });
                            break;
                        }
                    }
                }
            }

            loop {
                let y = rand::thread_rng().gen_range(0..ROOMS_Y) as usize;
                let x = rand::thread_rng().gen_range(0..ROOMS_X) as usize;
                if let Some(room) = &grid[y][x] {
                    if room.pos != position.room {
                        contents[y][x].push(Object {
                            hidden: false,
                            x: rand::thread_rng().gen_range(1..room.pos.w-1),
                            y: rand::thread_rng().gen_range(1..room.pos.h-1),
                            content: Content::Entrance
                        });
                        break;
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
        queue_hallways(&mut stdout, &hs_present, &hallways);

        let mut moved;
        let mut combat = Combat::new();
        let mut encounter = None;
        let mut enc_started = false;
        let mut enc_ended = false;
        let mut notification: Option<&str> = None;
        'lvl: loop {
            let y = (position.y *ROOMS_Y/height) as usize;
            let x = (position.x *ROOMS_X/width) as usize;

            // rendering -----------------------------------------------------------
            if enc_ended {
                queue_rect_cleanup(&mut stdout, &frame);
                for row in &grid {
                    for column in row {
                        if let Some(room) = column {
                            queue_rect(&mut stdout, &room.pos);
                        }
                    }
                }
                queue_hallways(&mut stdout, &hs_present, &hallways);
                combat = Combat::new();
                enc_ended = false;
                encounter = None;
            }

            queue_menu(&mut stdout, &player, width, height);
            if let Some(n) = notification {
                queue!(stdout, MoveTo((width-n.len()as u16)/2, height), Print(n)).unwrap();
                notification = None;
            }

            if let Some(i) = encounter {
                if enc_started {
                    queue_rect(&mut stdout, &frame);
                    enc_started = false;
                }
                let enemy = if let Some(room) = &grid[y][x] {
                    match (room.contents[i] as Object).content {
                        Content::Enemy(e) => e,
                        Content::Item(_) => unreachable!("cannot fight an item"),
                        Content::Entrance => unreachable!("cannot fight an entrance"),
                    }
                } else {unreachable!("should not be able to go outside of a room")};
                queue_enemy_encounter(&mut stdout, &frame, &enemy, &player, &combat);
            } else {
                for row in &grid {
                    for column in row {
                        if let Some(room) = column {
                            queue_room(&mut stdout, room, &position);
                        }
                    }
                }
                queue_position!(stdout, position);
            }
            stdout.flush().unwrap();
            // rendering -----------------------------------------------------------

            // events --------------------------------------------------------------
            moved = Move::NONE;
            if let Ok(e) = event::read() {
                match e {
                    Event::Key(k) => {
                        if let Some(_) = encounter {
                            match k.code {
                                KeyCode::Char('q') => {exited = true; break 'game;}
                                KeyCode::Char('1') => {combat.action = CMove::Attack}
                                KeyCode::Char('2') => {combat.action = CMove::Block}
                                KeyCode::Char('3') => {combat.action = CMove::Buff}
                                KeyCode::Char('4') => {combat.action = CMove::Dodge}
                                KeyCode::Char('5') => {combat.action = CMove::Run}
                                _ => {combat.action = CMove::NONE}
                            }
                        } else {
                            match k.code {
                                KeyCode::Char('q') => {exited = true; break 'game;}
                                KeyCode::Char('h') => {moved = Move::L}
                                KeyCode::Char('j') => {moved = Move::D}
                                KeyCode::Char('k') => {moved = Move::U}
                                KeyCode::Char('l') => {moved = Move::R}
                                _ => {moved = Move::NONE}
                            }
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
            let mut enemies_to_move = vec![];
            if moved != Move::NONE {
                if let Some(room) = &mut grid[y][x] {
                    let p_x = (position.x - position.room.x) as i16;
                    let p_y = (position.y - position.room.y) as i16;
                    for (o, obj) in room.contents.iter().enumerate() {
                        let o_x = obj.x as i16;
                        let o_y = obj.y as i16;
                        if o_x == p_x && o_y == p_y {
                            match obj.content {
                                Content::Item(item) => {
                                    match item.effect {
                                        Stat::NONE => unreachable!("`NONE` variant of `Stat` should not be accessable here"),
                                        Stat::__Count => unreachable!("`__Count` variant of `Stat` should never be constructed"),
                                        Stat::HP => { item_to_drop = Some(o);
                                            player.hp += item.value;
                                            notification = Some(PICKUP_HP)
                                        }
                                        Stat::DEF => { item_to_drop = Some(o);
                                            player.def += item.value;
                                            notification = Some(PICKUP_DEF)
                                        }
                                        Stat::ATK => { item_to_drop = Some(o);
                                            player.atk += item.value;
                                            notification = Some(PICKUP_ATK)
                                        }
                                        Stat::GOLD => { item_to_drop = Some(o);
                                            player.gold += item.value;
                                            notification = Some(PICKUP_GOLD)
                                        }
                                        Stat::EXP => { item_to_drop = Some(o);
                                            player.exp += item.value;
                                            notification = Some(PICKUP_EXP)
                                        }
                                    }
                                }
                                Content::Enemy(_) => {if encounter == None {enc_started = true; encounter = Some(o)}}
                                Content::Entrance => {break 'lvl;}
                            }
                        } else {
                            if let Content::Enemy(_) = &obj.content {
                                if random() || random() {
                                    let mut next_pos = Point {x: obj.x, y: obj.y};
                                    if p_y.abs_diff(o_y) > p_x.abs_diff(o_x) {
                                        match (p_y-o_y) > 0 {
                                            true => next_pos.y += 1,
                                            false => next_pos.y -= 1
                                        }
                                    } else {
                                        match (p_x-o_x) > 0 {
                                            true => next_pos.x += 1,
                                            false => next_pos.x -= 1
                                        }
                                    }
                                    enemies_to_move.push((o, next_pos));
                                }
                            }
                        }
                    }
                    for enemy in enemies_to_move {
                        // check for collisions with other objects
                        'once: loop {
                            for obj in &room.contents {
                                if obj.x == enemy.1.x && obj.y == enemy.1.y {
                                    break 'once;
                                }
                            }
                            queue_enemy_cleanup!(stdout, position, room.contents[enemy.0]);
                            room.contents[enemy.0].x = enemy.1.x;
                            room.contents[enemy.0].y = enemy.1.y;
                            if enemy.1.x == p_x as u16 && enemy.1.y == p_y as u16 {enc_started = true; encounter = Some(enemy.0)}
                            break 'once;
                        }
                    }
                    if let Some(o) = item_to_drop {
                        room.contents.remove(o);
                    }
                }
            }

            if combat.action != CMove::NONE {
                if let Some(i) = encounter {
                    'once: loop {
                        let room = if let Some(room) = &mut grid[y][x] {
                            room
                        } else {unreachable!("should not be able to go outside of a room")};
                        let enemy = match &mut room.contents[i].content {
                            Content::Enemy(e) => e,
                            Content::Item(_) => unreachable!("cannot fight an item"),
                            Content::Entrance => unreachable!("cannot fight an entrance"),
                        };

                        match combat.action {
                            CMove::NONE => {},
                            CMove::Attack => {if combat.buffs > 0 && player.atk - enemy.def/MULT_DEF > 0 {
                                enemy.hp -= player.atk - enemy.def/MULT_DEF
                            } else if combat.buffs == 0 && player.atk/MULT_ATK - enemy.def/MULT_DEF > 0 {
                                enemy.hp -= player.atk/MULT_ATK - enemy.def/MULT_DEF
                            }},
                            CMove::Block => {combat.blocks += player.def/10 +2},
                            CMove::Dodge => {combat.dodge = true},
                            CMove::Buff => {combat.buffs += player.lvl},
                            CMove::Run => {player.hp -= player.lvl; enc_ended = true; break 'once}
                        }

                        if enemy.hp <= 0 {
                            enc_ended = true;
                            match enemy.loot.effect {
                                Stat::NONE => unreachable!("enemy should not drop `NONE` loot"),
                                Stat::__Count => unreachable!("item `__Count` should never be constructed"),
                                Stat::HP => {player.hp += enemy.loot.value; notification = Some(PICKUP_HP)}
                                Stat::DEF => {player.def += enemy.loot.value; notification = Some(PICKUP_DEF)}
                                Stat::ATK => {player.atk += enemy.loot.value; notification = Some(PICKUP_ATK)}
                                Stat::EXP => {player.exp += enemy.loot.value; notification = Some(PICKUP_EXP)}
                                Stat::GOLD => {player.gold += enemy.loot.value; notification = Some(PICKUP_GOLD)}
                            }
                            room.contents.remove(i);
                            break 'once
                        }

                        if combat.dodge {}
                        else if combat.blocks > 0 && enemy.atk - player.def > 0{
                            player.hp -= enemy.atk - player.def
                        } else if combat.blocks == 0 && enemy.atk - player.def/MULT_DEF > 0{
                            player.hp -= enemy.atk - player.def/MULT_DEF
                        }

                        if combat.blocks > 0 {combat.blocks -= 1}
                        if combat.blocks > player.lvl {combat.blocks = player.lvl}
                        if combat.buffs > 0 {combat.buffs -= 1}
                        if combat.buffs > player.lvl {combat.buffs = player.lvl}
                        combat.dodge = false;
                        combat.action = CMove::NONE;
                        break 'once;
                    }
                }
            }

            if player.hp <= 0 {died = true; break 'game}
            if player.exp >= 20 {
                player.lvl += 1;
                player.hp += 5;
                player.def += 1;
                player.atk += 1;
                player.exp -= 20;
                notification = Some(LEVELUP);
            }
            // logic ---------------------------------------------------------------
        }
        execute!(stdout, LeaveAlternateScreen).unwrap();
    }

    execute!(stdout, LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();

    if exited {println!("You have exited having achieved level: {}", player.lvl)}
    else if died {println!("You have died having achieved level: {}", player.lvl)}
    Ok(())
}

const ENCOUNTER_MSG: &str = "Fighting the ";
const PLAYER: &str = " Player: ";
const MENU1: &str = "[1] Attack";
const MENU2: &str = "[2] Block";
const MENU3: &str = "[3] Buff up";
const MENU4: &str = "[4] Dodge";
const MENU5: &str = "[5] Run";
const LVL2: &str = " (lvl 2+)";
fn queue_enemy_encounter(stdout: &mut Stdout, frame: &Rect, enemy: &Enemy, player: &Player, combat: &Combat) {
    let str = enemy.kind.to_str();
    let line = "-".repeat((frame.w as usize-PLAYER.len())/2);
    let pl_pad = " ".repeat((frame.w as usize-20)/4);
    let c_pad = " ".repeat((frame.w as usize-20)/3);

    // TODO: automatic padding
    queue!(stdout,
           MoveTo(frame.x +(frame.w -ENCOUNTER_MSG.len()as u16 -str.len()as u16)/2, frame.y+1),
           Print(format!("{}{}", ENCOUNTER_MSG, str)),

           MoveTo(frame.x +(frame.w-16)/2, frame.y+2),
           Print(format!("atk: {}, def: {}", enemy.atk, enemy.def)),
           MoveTo(frame.x +(frame.w-11)/2, frame.y+3),
           Print(format!("hp left: {}  ", enemy.hp)),

           MoveTo(frame.x +(frame.w-11)/2, frame.y+frame.h-9),
           Print(format!("{}", MENU1)),
           MoveTo(frame.x +(frame.w-11)/2, frame.y+frame.h-8),
           Print(format!("{}", MENU2)),
           MoveTo(frame.x +(frame.w-11)/2, frame.y+frame.h-7),
           Print(format!("{}", MENU3)),
           Print(format!("{}", if player.lvl < 2 {
               LVL2
           } else {""})),

           MoveTo(frame.x +(frame.w-11)/2, frame.y+frame.h-6),
           Print(format!("{}", MENU4)),
           MoveTo(frame.x +(frame.w-11)/2, frame.y+frame.h-5),
           Print(format!("{}", MENU5)),

           MoveTo(frame.x+1, frame.y+frame.h-4),
           Print(format!("{}{}{}", line, PLAYER, "-".repeat(frame.w as usize-line.len() -PLAYER.len()-2))),
           MoveTo(frame.x+1, frame.y+frame.h-3),
           Print(format!("{c_pad}blocks: {}{c_pad}buff ups: {}  ", combat.blocks, combat.buffs)),
           MoveTo(frame.x+1, frame.y+frame.h-2),
           Print(format!("{pl_pad}hp: {}{pl_pad}def: {}{pl_pad}atk: {}   ", player.hp, player.def, player.atk)),
          ).unwrap();
}

fn add_hallway(hs_present: &mut [bool], hallways: &mut [Hallway], doors_count: &mut usize,
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
}

fn queue_hallways(stdout: &mut Stdout, hs_present: &[bool], hallways: &[Hallway]) {
    for i in 0..HALLWAYS_SIZE {
        if !hs_present[i] {break}
        else {
            queue!(stdout,
                   SetAttribute(Attribute::Bold),
                   SetBackgroundColor(Color::Cyan),
                   SetForegroundColor(Color::Black),
                   MoveTo(hallways[i].entr[0].x, hallways[i].entr[0].y), Print(format!("{}", i+1)),
                   if i < 9 {
                       MoveTo(hallways[i].entr[1].x, hallways[i].entr[1].y)
                   } else {
                       MoveTo(hallways[i].entr[1].x-1, hallways[i].entr[1].y)
                   }, Print(format!("{}", i+1)),
                   SetAttribute(Attribute::Reset),
                   ResetColor).unwrap();
        }
    }
}

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

fn queue_rect_cleanup(stdout: &mut Stdout, rect: &Rect) {
    for y in 0..rect.h{
        queue!(stdout,
               MoveTo(rect.x, rect.y+y),
               Print(" ".repeat(rect.w.into()))
              ).unwrap();
    }
}

fn queue_room(stdout: &mut Stdout, room: &Room, position: &Position) {
    for obj in &room.contents {
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
                Content::Entrance => queue!(stdout, Print(CHAR_ENTRANCE)).unwrap()
            }
        }
    }
}
