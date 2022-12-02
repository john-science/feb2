/*
February Second
 */
// Import Std Libs
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

// Import Third-Party
use tcod::colors::*;
use tcod::console::*;
use tcod::input::{self, Event, Key};
use tcod::map::{Map as FovMap};

// Import Locally
mod ai_algos;
mod constants;
mod equipment;
mod loot_table;
mod magic;
mod map;
mod menus;
mod moves;
mod npc_table;
mod objects;
mod player;
mod transition;
mod ui;
mod utils;
use ai_algos::ai_take_turn;
use constants::AUTHOR_LINE;
use constants::CHARACTER_SCREEN_WIDTH;
use constants::FONT_IMG;
use constants::GAME_TITLE;
use constants::KARMA_TO_ASCEND;
use constants::LIMIT_FPS;
use constants::MAP_HEIGHT;
use constants::MAP_WIDTH;
use constants::MAX_LVL;
use constants::PANEL_HEIGHT;
use constants::PLAYER;
use constants::SAVE_FILE;
use constants::SCREEN_HEIGHT;
use constants::SCREEN_WIDTH;
use equipment::drop_item;
use equipment::pick_item_up;
use equipment::player_use_item;
use map::make_map;
use map::Map;
use menus::inventory_menu;
use menus::menu;
use menus::Messages;
use menus::msgbox;
use menus::Tcod;
use moves::player_move_or_attack;
use objects::Equipment;
use objects::Fighter;
use objects::Game;
use objects::Item;
use objects::Object;
use objects::Slot;
use player::level_up;
use player::xp_to_level_up;
use ui::render_all;

// TODO: Game.map should be game.maps. Step 1 on the way to building down stairs.
// TODO: The color of potions, or maybe the font, is hard to read.

// Advance to the next level
fn next_level(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) -> bool {
    if game.map_level == MAX_LVL {
        if objects[PLAYER].fighter.as_ref().unwrap().karma >= KARMA_TO_ASCEND {
            game.messages.add(
                "You ascend from Purgatory.",
                RED,
            );
            return false;
        } else {
            game.messages.add(
                "Your karma is too low to leave Purgatory.",
                RED,
            );
            return false;
        }
    } else {
        game.messages.add(
            "You ascend higher into Purgatory...",
            RED,
        );
        game.map_level += 1;
        game.map = make_map(objects, game.map_level);
        initialise_fov(tcod, &game.map);
    }
    return true;
}


#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


// TODO: Key "m" should open a scrollable messages window.
// TODO: Hitting "?" should pop up a command menu. (Could we make this more configurable/automatic?)
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

        // go up stairs, if the player is on them
        (Key { code: Text, .. }, ">", true) => {
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
            if player_on_stairs {
                let success: bool = next_level(tcod, game, objects);
                if success {
                    // TODO: If game.level > MAX_LVL: return WinExit
                    return TookTurn;
                } else {
                    return DidntTakeTurn;
                }
            }
            return DidntTakeTurn;
        }

        // TODO: Combine inventory and character stuff?
        (Key { code: Text, .. }, "c", true) => {
            // show character information
            let player = &objects[PLAYER];
            let level = player.level;
            let level_up_xp = xp_to_level_up(player.level);
            if let Some(fighter) = player.fighter.as_ref() {
                let msg = format!(
"Character information

Karma: {}
Level: {}
Experience: {} of {}

Maximum HP: {}
Attack: {}
Defense: {}",
                    fighter.karma,
                    level,
                    fighter.xp,
                    level_up_xp,
                    fighter.max_hp(),
                    fighter.power(),
                    fighter.defense(),
                );
                msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }

            return DidntTakeTurn;
        }

        // show the inventory
        (Key { code: Text, .. }, "i", true) => {
            let player = &mut objects[PLAYER];
            if let Some(fighter) = player.fighter.as_mut() {
                // show the inventory: if an item is selected, use it
                let inventory_index = inventory_menu(
                    &fighter.inventory,
                    "Press the key next to an item to use it, or any other to cancel.\n",
                    &mut tcod.root,
                );
                if let Some(inventory_index) = inventory_index {
                    player_use_item(inventory_index, tcod, game, objects);
                }
            }
            return DidntTakeTurn;
        }

        // Pick up an item
        (Key { code: Text, .. }, "g", true) => {
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(item_id, PLAYER, &mut game.messages, objects);
                return TookTurn;
            } else {
                return DidntTakeTurn;
            }
        }

        // show the inventory; if an item is selected, drop it
        (Key { code: Text, .. }, "d", true) => {
            let player = &objects[PLAYER];
            if let Some(fighter) = player.fighter.as_ref() {
                let inventory_index = inventory_menu(
                    &fighter.inventory,
                    "Press the key next to an item to drop it, or any other to cancel.\n'",
                    &mut tcod.root,
                );
                if let Some(inventory_index) = inventory_index {
                    drop_item(inventory_index, PLAYER, &mut game.messages, objects);
                }
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


// NOTE: There can currently be only one save game at a time.
//       But the save games are human-readable, storable, and editable.
fn save_game(game: &Game, objects: &[Object]) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game, objects))?;
    let mut file = File::create(SAVE_FILE)?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}


fn load_game() -> Result<(Game, Vec<Object>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open(SAVE_FILE)?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Object>)>(&json_save_state)?;
    Ok(result)
}


fn new_game(tcod: &mut Tcod) -> (Game, Vec<Object>) {
    // create object representing the player
    let mut player = Object::new(0, 0, '@', "you", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter::new(100, 2, 3, 0, false));

    // initial equipment: a dagger
    let mut dagger = Object::new(0, 0, '-', "dagger", SKY, false);
    dagger.item = Some(Item::Sword);
    dagger.equipment = Some(Equipment {
        equipped: true,
        slot: Slot::OffHand,
        max_hp_bonus: 0,
        defense_bonus: 0,
        power_bonus: 2,
    });
    player.fighter.as_mut().unwrap().inventory.push(dagger);

    // the list of objects with those two
    let mut objects = vec![player];

    // make a Map of room objects
    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects, 1),
        messages: Messages::new(),
        map_level: 1,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

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


fn load_version_equals(tcod: &mut Tcod, game: &Game) -> bool {
    let this_version: String = env!("CARGO_PKG_VERSION").to_string();

    if !game.version.eq(&this_version) {
        let mut load_err: String = "ERROR Loading Game\n\nCannot load save game, because it is the wrong version.\n".to_string();
        load_err.push_str("\nsave game version: ");
        load_err.push_str(&game.version);
        load_err.push_str("\nload game version: ");
        load_err.push_str(&this_version);

        msgbox(&load_err, 32, &mut tcod.root);

        return false;
    } else {
        return true;
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
            GAME_TITLE,
        );
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            AUTHOR_LINE,
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
                        if !load_version_equals(tcod, &game) {
                            continue;
                        } else {
                            initialise_fov(tcod, &game.map);
                            play_game(tcod, &mut game, &mut objects);
                        }
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
        .font(FONT_IMG, FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title(GAME_TITLE)
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
