/*
February Second
 */
// Import Std Libs
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

// Import Third-Party
use rand::Rng;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::{self, Event, Key};
use tcod::map::{Map as FovMap};

// Import Locally
mod ai_algos;
mod constants;
mod equipment;
mod magic;
mod map;
mod menus;
mod moves;
mod objects;
mod transition;
mod ui;
mod utils;
use ai_algos::ai_take_turn;
use constants::CHARACTER_SCREEN_WIDTH;
use constants::MAP_HEIGHT;
use constants::MAP_WIDTH;
use constants::PANEL_HEIGHT;
use constants::PLAYER;
use constants::SCREEN_HEIGHT;
use constants::SCREEN_WIDTH;
use equipment::drop_item;
use equipment::pick_item_up;
use equipment::use_item;
use map::create_h_tunnel;
use map::create_room;
use map::create_v_tunnel;
use map::Map;
use map::Rect;
use map::Tile;
use menus::inventory_menu;
use menus::menu;
use menus::Messages;
use menus::msgbox;
use menus::Tcod;
use moves::is_blocked;
use moves::player_move_or_attack;
use objects::Ai;
use objects::DeathCallback;
use objects::Equipment;
use objects::Fighter;
use objects::Game;
use objects::Item;
use objects::Object;
use objects::Slot;
use transition::from_map_level;
use transition::Transition;
use ui::render_all;

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// parameters for map generator
const ROOM_MAX_SIZE: i32 = 12;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 32;

// experience and level-ups (BASE + level * FACTOR)
const LEVEL_UP_BASE: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;
const LEVEL_SCREEN_WIDTH: i32 = 40;


// TODO: Break this into multiple files.
// TODO: The color of potions, or maybe the font, is hard to read.
// TODO: I would like to have item/NPC/player data in data files that are ingested at compile time.


fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>, level: u32) {
    use rand::distributions::{IndependentSample, Weighted, WeightedChoice};

    // TODO: Move NPC table to tables.rs
    // TODO: Switch from "npc" to "npc
    // maximum number of npcs per room
    let max_npcs = from_map_level(
        &[
            Transition { level: 1, value: 2 },
            Transition { level: 4, value: 3 },
            Transition { level: 6, value: 5 },
        ],
        level,
    );

    // choose random number of npcs
    let num_npcs = rand::thread_rng().gen_range(0, max_npcs + 1);

    // npc random table
    let troll_chance = from_map_level(
        &[
            Transition {
                level: 3,
                value: 15,
            },
            Transition {
                level: 5,
                value: 30,
            },
            Transition {
                level: 7,
                value: 60,
            },
        ],
        level,
    );

    // TODO: Move NPC table to tables.rs
    let npc_chances = &mut [  // TODO: Change name to npc_table
        Weighted {
            weight: 80,
            item: "orc",  // TODO: const NPC_ORC: i32 = 0;
        },
        Weighted {
            weight: troll_chance,
            item: "troll",
        },
    ];

    for _ in 0..num_npcs {
        // choose random spot for this npc
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // only place it if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            let npc_choice = WeightedChoice::new(npc_chances);

            let mut npc = match npc_choice.ind_sample(&mut rand::thread_rng()) {
                "orc" => {
                    // create an orc
                    let mut orc = Object::new(x, y, 'O', "orc", DESATURATED_GREEN, true);
                    orc.ai = Some(Ai::Basic);
                    orc.fighter = Some(Fighter {
                        base_max_hp: 20,
                        hp: 20,
                        base_defense: 0,
                        base_power: 4,
                        xp: 35,
                        on_death: DeathCallback::Npc,
                    });
                    orc
                }
                "troll" => {
                    // create a troll
                    let mut troll = Object::new(x, y, 'T', "troll", DARKER_GREEN, true);
                    troll.ai = Some(Ai::Basic);
                    troll.fighter = Some(Fighter {
                        base_max_hp: 30,
                        hp: 30,
                        base_defense: 2,
                        base_power: 8,
                        xp: 100,
                        on_death: DeathCallback::Npc,
                    });
                troll
                }
                _ => unreachable!(),
            };

            npc.alive = true;
            npc.ai = Some(Ai::Basic);
            objects.push(npc);
        }
    }

    // maximum number of items per room
    let max_items = from_map_level(
        &[
            Transition { level: 1, value: 1 },
            Transition { level: 4, value: 2 },
        ],
        level,
    );

    // TODO: move loot table to tables.rs
    // item random table
    let item_chances = &mut [
        Weighted {
            weight: from_map_level(&[Transition { level: 4, value: 5 }], level),
            item: Item::Sword,
        },
        Weighted {
            weight: from_map_level(
                &[Transition {
                    level: 8,
                    value: 15,
                }],
                level,
            ),
            item: Item::Shield,
        },
        Weighted {
            weight: 35,
            item: Item::Heal,
        },
        Weighted {
            weight: from_map_level(
                &[Transition {
                    level: 4,
                    value: 25,
                }],
                level,
            ),
            item: Item::Lightning,
        },
        Weighted {
            weight: from_map_level(
                &[Transition {
                    level: 6,
                    value: 25,
                }],
                level,
            ),
            item: Item::Fireball,
        },
        Weighted {
            weight: from_map_level(
                &[Transition {
                    level: 2,
                    value: 10,
                }],
                level,
            ),
            item: Item::Confuse,
        },
    ];
    let item_choice = WeightedChoice::new(item_chances);

    // choose random number of items
    let num_items = rand::thread_rng().gen_range(0, max_items + 1);

    for _ in 0..num_items {
        // choose random spot for this item
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // only place it if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            let item = match item_choice.ind_sample(&mut rand::thread_rng()) {
                Item::Shield => {
                    // create a shield
                    let mut object = Object::new(x, y, '[', "shield", DARKER_ORANGE, false);
                    object.item = Some(Item::Shield);
                    object.equipment = Some(Equipment {
                        equipped: false,
                        slot: Slot::LeftHand,
                        max_hp_bonus: 0,
                        defense_bonus: 1,
                        power_bonus: 0,
                    });
                    object
                }
                Item::Sword => {
                    // create a sword
                    let mut object = Object::new(x, y, '/', "sword", SKY, false);
                    object.item = Some(Item::Sword);
                    object.equipment = Some(Equipment {
                        equipped: false,
                        slot: Slot::RightHand,
                        max_hp_bonus: 0,
                        defense_bonus: 0,
                        power_bonus: 3,
                    });
                    object
                }
                Item::Heal => {
                    // create a healing potion
                    let mut object = Object::new(x, y, '!', "healing potion", VIOLET, false);
                    object.item = Some(Item::Heal);
                    object
                }
                Item::Lightning => {
                    // create a lightning bolt scroll
                    let mut object =
                        Object::new(x, y, '#', "scroll of lightning bolt", LIGHT_YELLOW, false);
                    object.item = Some(Item::Lightning);
                    object
                }
                Item::Fireball => {
                    // create a fireball scroll
                    let mut object =
                        Object::new(x, y, '#', "scroll of fireball", LIGHT_YELLOW, false);
                    object.item = Some(Item::Fireball);
                    object
                }
                Item::Confuse => {
                    // create a confuse scroll
                    let mut object =
                        Object::new(x, y, '#', "scroll of confusion", LIGHT_YELLOW, false);
                    object.item = Some(Item::Confuse);
                    object
                }
            };

            objects.push(item);
        }
    }
}


fn level_up(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    let player = &mut objects[PLAYER];
    let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
    // see if the player's experience is enough to level-up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        player.level += 1;
        game.messages.add(
            format!(
                "Your battle skills grow stronger! You reached level {}!",
                player.level
            ),
            YELLOW,
        );

        // Let the player choose a stat to level up
        let fighter = player.fighter.as_mut().unwrap();
        let mut choice = None;
        while choice.is_none() {
            // keep asking until a choice is made
            choice = menu(
                "Level up! Choose a stat to raise:\n",
                &[
                    format!("Constitution (+20 HP, from {})", fighter.base_max_hp),
                    format!("Strength (+1 attack, from {})", fighter.base_power),
                    format!("Agility (+1 defense, from {})", fighter.base_defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut tcod.root,
            );
        }
        fighter.xp -= level_up_xp;
        match choice.unwrap() {
            0 => {
                fighter.base_max_hp += 20;
                fighter.hp += 20;
            }
            1 => {
                fighter.base_power += 1;
            }
            2 => {
                fighter.base_defense += 1;
            }
            _ => unreachable!(),
        }
    }
}


// TODO: Bug. I entered a game and there were 3 npcs in the start room. (No npcs should spawn in FOV of the player at the Start.)
fn make_map(objects: &mut Vec<Object>, level: u32) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // TODO: Player cannot visit old levels! OMG, this deletes everything!!!!!!
    // Player is the first element, remove everything else.
    // NOTE: works only when the player is the first object!
    assert_eq!(&objects[PLAYER] as *const Object, &objects[0] as *const Object);
    objects.truncate(1);

    // generate a random set of roooms
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }

                // add some content to this room, such as npcs
                place_objects(new_room, &map, objects, level);
            }

            // finally, append the new room to the list
            rooms.push(new_room);
        }
    }

    // create stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut stairs = Object::new(last_room_x, last_room_y, '<', "stairs", WHITE, false);  // TODO: Flip!
    stairs.always_visible = true;
    objects.push(stairs);

    return map;
}


// Advance to the next level
fn next_level(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    game.messages.add(
        "You descend deeper into Purgatory...",  // TODO: Flip!
        RED,
    );
    game.map_level += 1;
    game.map = make_map(objects, game.map_level);
    initialise_fov(tcod, &game.map);
}


#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


// TODO: Fullscreen isn't working.
fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) -> PlayerAction {
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;
    match (tcod.key, tcod.key.text(), player_alive) {
        // movement keys
        (Key { code: Up, .. }, _, true) => {
            player_move_or_attack(0, -1, game, objects);
            return TookTurn;
        }
        (Key { code: Down, .. }, _, true) => {
            player_move_or_attack(0, 1, game, objects);
            return TookTurn;
        }
        (Key { code: Left, .. }, _, true) => {
            player_move_or_attack(-1, 0, game, objects);
            return TookTurn;
        }
        (Key { code: Right, .. }, _, true) => {
            player_move_or_attack(1, 0, game, objects);
            return TookTurn;
        }
        // numpad keys
        (Key { code: NumPad1, .. }, _, true) | (Key { code: End, .. }, _, true) => {
            player_move_or_attack(-1, 1, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad2, .. }, _, true) => {
            player_move_or_attack(0, 1, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad3, .. }, _, true) | (Key { code: PageDown, .. }, _, true) => {
            player_move_or_attack(1, 1, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad4, .. }, _, true) => {
            player_move_or_attack(-1, 0, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad5, .. }, _, true) => {
            return TookTurn;
        }
        (Key { code: NumPad6, .. }, _, true) => {
            player_move_or_attack(1, 0, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad7, .. }, _, true) | (Key { code: Home, .. }, _, true) => {
            player_move_or_attack(-1, -1, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad8, .. }, _, true) => {
            player_move_or_attack(0, -1, game, objects);
            return TookTurn;
        }
        (Key { code: NumPad9, .. }, _, true) | (Key { code: PageUp, .. }, _, true) => {
            player_move_or_attack(1, -1, game, objects);
            return TookTurn;
        }

        // go down stairs, if the player is on them
        (Key { code: Text, .. }, "<", true) => {  // TODO: Flip!
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
            if player_on_stairs {
                next_level(tcod, game, objects);
            }
            return DidntTakeTurn;
        }

        // TODO: Maybe make an XP bar?
        // TODO: Combine inventory and character stuff?
        (Key { code: Text, .. }, "c", true) => {
            // show character information
            let player = &objects[PLAYER];
            let level = player.level;
            let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
            if let Some(fighter) = player.fighter.as_ref() {
                let msg = format!(
"Character information

Level: {}
Experience: {}
Experience to level up: {}

Maximum HP: {}
Attack: {}
Defense: {}",
                    level,
                    fighter.xp,
                    level_up_xp,
                    player.max_hp(game),
                    player.power(game),
                    player.defense(game),
                );
                msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }

            return DidntTakeTurn;
        }

        // show the inventory
        (Key { code: Text, .. }, "i", true) => {
            // show the inventory: if an item is selected, use it
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next to an item to use it, or any other to cancel.\n",
                &mut tcod.root,
            );
            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, objects);
            }
            return DidntTakeTurn;
        }

        // Pick up an item
        (Key { code: Text, .. }, "g", true) => {
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(item_id, game, objects);
                return TookTurn;
            } else {
                return DidntTakeTurn;
            }
        }

        // show the inventory; if an item is selected, drop it
        (Key { code: Text, .. }, "d", true) => {
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next to an item to drop it, or any other to cancel.\n'",
                &mut tcod.root,
            );
            if let Some(inventory_index) = inventory_index {
                drop_item(inventory_index, game, objects);
            }
            return DidntTakeTurn;
        }

        // Escape to exit game
        (Key { code: Escape, .. }, _, _) => { return Exit; }

        (_, _, _) => { return DidntTakeTurn; }
    };
}


fn initialise_fov(tcod: &mut Tcod, map: &Map) {
    // create the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !map[x as usize][y as usize].block_sight,
                !map[x as usize][y as usize].blocked,
            );
        }
    }

    // unexplored areas start black (which is the default background color)
    tcod.con.clear();
}


fn save_game(game: &Game, objects: &[Object]) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game, objects))?;
    let mut file = File::create("savegame")?;  // TODO: Default savegame file only?
    file.write_all(save_data.as_bytes())?;
    Ok(())
}


fn load_game() -> Result<(Game, Vec<Object>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Object>)>(&json_save_state)?;
    Ok(result)
}


fn new_game(tcod: &mut Tcod) -> (Game, Vec<Object>) {
    // create object representing the player
    let mut player = Object::new(0, 0, '@', "you", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter {
        base_max_hp: 100,  // TODO: These numbers seem like they should be constants, or config?
        hp: 100,
        base_defense: 2,
        base_power: 3,
        xp: 0,
        on_death: DeathCallback::Player,
    });

    // the list of objects with those two
    let mut objects = vec![player];

    // make a Map of room objects
    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects, 1),
        messages: Messages::new(),
        inventory: vec![],
        map_level: 1,
    };

    // initial equipment: a dagger
    let mut dagger = Object::new(0, 0, '-', "dagger", SKY, false);
    dagger.item = Some(Item::Sword);
    dagger.equipment = Some(Equipment {
        equipped: true,
        slot: Slot::LeftHand,
        max_hp_bonus: 0,
        defense_bonus: 0,
        power_bonus: 2,
    });
    game.inventory.push(dagger);

    initialise_fov(tcod, &game.map);

    // a welcome message
    game.messages.add(
        "Welcome stranger! Ascend from Purgatory, or be stuck here forever.",
        RED,
    );

    return (game, objects);
}


fn play_game(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // the game loop!
    while !tcod.root.window_closed() {
        // clear the off-screen console
        tcod.con.clear();

        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }

        // render the screen
        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(tcod, game, objects, fov_recompute);

        tcod.root.flush();

        // level up if needed
        level_up(tcod, game, objects);

        // handle keys and exit game if needed
        previous_player_position = objects[PLAYER].pos();
        let player_action = handle_keys(tcod, game, objects);
        if player_action == PlayerAction::Exit {
            save_game(game, objects).unwrap();
            break;
        }

        // let npcs take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if id != PLAYER && objects[id].ai.is_some() {
                    ai_take_turn(id, tcod, game, objects);
                }
            }
        }
    }
}


fn main_menu(tcod: &mut Tcod) {
    let img = tcod::image::Image::from_file("menu_background.png")
        .ok()
        .expect("Background image not found");

    while !tcod.root.window_closed() {
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        tcod.root.set_default_foreground(LIGHT_YELLOW);
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 4,
            BackgroundFlag::None,
            TextAlignment::Center,
            "February Second",
        );
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            "By John Science",
        );

        // show options and wait for the player's choice
        let choices = &["Play a new game", "Continue last game", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {
            Some(0) => {
                // new game
                let (mut game, mut objects) = new_game(tcod);
                play_game(tcod, &mut game, &mut objects);
            }
            Some(1) => {
                // load game
                match load_game() {
                    Ok((mut game, mut objects)) => {
                        initialise_fov(tcod, &game.map);
                        play_game(tcod, &mut game, &mut objects);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load.\n", 24, &mut tcod.root);
                        continue;
                    }
                }
            }
            Some(2) => {
                // quit
                break;
            }
            _ => {}
        }
    }
}


fn main() {
    // set the FPS
    tcod::system::set_fps(LIMIT_FPS);

    // initialize the TCOD "Root" object
    let root: Root = Root::initializer()
        .font("dejavu16x16.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("February Second")
        .init();

    // use the TCOD "Root" object to create a mutable TCOD struct
    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        key: Default::default(),
        mouse: Default::default(),
    };

    main_menu(&mut tcod);
}
