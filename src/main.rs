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
//use tcod::tileset::{load_tilesheet};

// Import Locally
mod ai_algos;
mod constants;
mod equipment;
mod loot_table;
mod magic;
mod map;
mod map_algos;
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
use constants::FONT_IMG;
use constants::GAME_TITLE;
use constants::KARMA_TO_ASCEND;
use constants::LIMIT_FPS;
use constants::MAP_HEIGHT;
use constants::MAP_WIDTH;
use constants::NUM_LVLS;
use constants::PANEL_HEIGHT;
use constants::PLAYER;
use constants::SAVE_FILE;
use constants::SCREEN_HEIGHT;
use constants::SCREEN_WIDTH;
use equipment::drop_item;
use equipment::pick_item_up;
use equipment::player_use_item;
use map::Map;
use menus::help_menu;
use menus::inventory_menu;
use menus::menu;
use menus::msgbox;
use menus::Tcod;
use moves::player_move_or_attack;
use moves::PlayerAction;
use objects::Fighter;
use objects::Game;
use objects::Object;
use player::character_screen;
use player::level_up;
use player::reincarnate_reset;
use ui::render_all;


fn change_player_level(objects: &mut Vec<Vec<Object>>, from_lvl: usize, to_lvl: usize) {
    let player = objects[from_lvl].swap_remove(PLAYER);

    let mut tmp: Vec<Object> = vec![player];
    tmp.extend(objects[to_lvl].drain(0..));
    objects[to_lvl] = tmp;
}


fn go_up_level(tcod: &mut Tcod, game: &mut Game, all_objects: &mut Vec<Vec<Object>>) -> bool {
    if game.lvl == (NUM_LVLS as usize - 1) {
        if all_objects[game.lvl][PLAYER].fighter.as_ref().unwrap().karma >= KARMA_TO_ASCEND {
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
        game.lvl += 1;
        change_player_level(all_objects, game.lvl - 1, game.lvl);
        all_objects[game.lvl][PLAYER].x = game.down_stairs[game.lvl].0;
        all_objects[game.lvl][PLAYER].y = game.down_stairs[game.lvl].1;
        initialise_fov(tcod, &game.map());
    }
    return true;
}


fn go_down_level(tcod: &mut Tcod, game: &mut Game, all_objects: &mut Vec<Vec<Object>>) -> bool {
    if game.lvl == 0 {
        game.messages.add(
            "There is no going lower than where you are.",
            RED,
        );
        return false;
    } else {
        game.messages.add(
            "You descend back down into Purgatory.",
            RED,
        );
        game.lvl -= 1;
        change_player_level(all_objects, game.lvl + 1, game.lvl);
        all_objects[game.lvl][PLAYER].x = game.up_stairs[game.lvl].0;
        all_objects[game.lvl][PLAYER].y = game.up_stairs[game.lvl].1;
        initialise_fov(tcod, &game.map());
    }
    return true;
}


// TODO: Key "m" should open a scrollable messages window.
// TODO: Fullscreen isn't working.
fn handle_keys(tcod: &mut Tcod, game: &mut Game, all_objects: &mut Vec<Vec<Object>>) -> PlayerAction {
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let objects = &mut all_objects[game.lvl];
    let player_alive = objects[PLAYER].alive;
    match (tcod.key, tcod.key.text(), player_alive) {
        // movement keys
        (Key { code: Up, .. }, _, true) => {
            return player_move_or_attack(0, -1, game, objects);
        }
        (Key { code: Down, .. }, _, true) => {
            return player_move_or_attack(0, 1, game, objects);
        }
        (Key { code: Left, .. }, _, true) => {
            return player_move_or_attack(-1, 0, game, objects);
        }
        (Key { code: Right, .. }, _, true) => {
            return player_move_or_attack(1, 0, game, objects);
        }
        // numpad keys
        (Key { code: NumPad1, .. }, _, true) | (Key { code: End, .. }, _, true) => {
            return player_move_or_attack(-1, 1, game, objects);
        }
        (Key { code: NumPad2, .. }, _, true) => {
            return player_move_or_attack(0, 1, game, objects);
        }
        (Key { code: NumPad3, .. }, _, true) | (Key { code: PageDown, .. }, _, true) => {
            return player_move_or_attack(1, 1, game, objects);
        }
        (Key { code: NumPad4, .. }, _, true) => {
            return player_move_or_attack(-1, 0, game, objects);
        }
        (Key { code: NumPad5, .. }, _, true) => {
            return TookTurn;
        }
        (Key { code: NumPad6, .. }, _, true) => {
            return player_move_or_attack(1, 0, game, objects);
        }
        (Key { code: NumPad7, .. }, _, true) | (Key { code: Home, .. }, _, true) => {
            return player_move_or_attack(-1, -1, game, objects);
        }
        (Key { code: NumPad8, .. }, _, true) => {
            return player_move_or_attack(0, -1, game, objects);
        }
        (Key { code: NumPad9, .. }, _, true) | (Key { code: PageUp, .. }, _, true) => {
            return player_move_or_attack(1, -1, game, objects);
        }

        // go up stairs, if the player is on them
        (Key { code: Text, .. }, ">", true) => {
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "up-stairs");
            if player_on_stairs {
                let success: bool = go_up_level(tcod, game, all_objects);
                if success {
                    // TODO: If game.level >= NUM_LVLS: return WinExit
                    return TookTurn;
                } else {
                    return DidntTakeTurn;
                }
            }
            return DidntTakeTurn;
        }

        // go down stairs, if the player is on them
        (Key { code: Text, .. }, "<", true) => {
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "down-stairs");
            if player_on_stairs {
                let success: bool = go_down_level(tcod, game, all_objects);
                if success {
                    return TookTurn;
                } else {
                    return DidntTakeTurn;
                }
            }
            return DidntTakeTurn;
        }

        (Key { code: Text, .. }, "c", true) => {
            let player = &objects[PLAYER];
            character_screen(tcod, player, game);
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
            return MenuAction;
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

        // Help Menu
        (Key { code: Text, .. }, "?", true) => {
            help_menu(tcod);
            return DidntTakeTurn;
        }

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
fn save_game(game: &Game,
             objects: &Vec<Vec<Object>>,
             start_objects: &Vec<Vec<Object>>
            ) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game, objects, start_objects))?;
    let mut file = File::create(SAVE_FILE)?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}


fn load_game() -> Result<(Game, Vec<Vec<Object>>, Vec<Vec<Object>>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open(SAVE_FILE)?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Vec<Object>>, Vec<Vec<Object>>)>(&json_save_state)?;
    Ok(result)
}


fn reincarnate(game: &mut Game,
               objects: &mut Vec<Vec<Object>>,
               start_objects: &Vec<Vec<Object>>) {
    // copy over objects
    let mut player: Object = objects[game.lvl][0].clone();
    reincarnate_reset(&mut player);
    (player.x, player.y) = game.down_stairs[0];
    *objects = start_objects.clone();
    objects[0][PLAYER] = player;

    // update game
    game.lvl = 0;
    game.day += 1;
    game.turn += 1;
}


fn new_game(tcod: &mut Tcod) -> (Game, Vec<Vec<Object>>, Vec<Vec<Object>>) {
    // create object representing the player
    let mut player = Object::new(0, 0, '@', "you", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter::new(100, 2, 3, 0, false));

    // NOTE: This is here to ensure the first rebirth matches all following reincarnations
    reincarnate_reset(&mut player);

    // the list of objects in the game, by floor
    let mut objects: Vec<Vec<Object>> = vec![vec![]; NUM_LVLS as usize];
    objects[0].push(player);

    let mut game = Game::new(&mut objects);

    initialise_fov(tcod, &game.map());

    // a welcome message
    game.messages.add(
        "You wake up in Purgatory. Ascend from this place, or be stuck here forever.",
        RED,
    );

    // save off the starting position of the game
    let starts = objects.clone();

    return (game, objects, starts);
}


fn play_game(tcod: &mut Tcod, game: &mut Game,
            all_objects: &mut Vec<Vec<Object>>,
            start_objects: &mut Vec<Vec<Object>>
            ) {
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);
    let mut last_action: PlayerAction = PlayerAction::DidntTakeTurn;

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
        let lvl: usize = game.lvl as usize;
        let fov_recompute = previous_player_position != (all_objects[lvl][PLAYER].pos());
        render_all(tcod, game, &mut all_objects[lvl], fov_recompute);

        // handle death and reincarnation
        if !all_objects[lvl][PLAYER].alive {
            last_action = PlayerAction::DidntTakeTurn;
            character_screen(tcod, &all_objects[lvl][PLAYER], game);
            reincarnate(game, all_objects, start_objects);
            initialise_fov(tcod, &game.map());
            tcod.root.flush();
            continue;
        }

        tcod.root.flush();

        if last_action == PlayerAction::MenuAction {
            last_action = PlayerAction::DidntTakeTurn;
            continue;
        }

        // level up if needed
        level_up(tcod, game, &mut all_objects[lvl]);

        // handle keys and exit game if needed
        previous_player_position = all_objects[lvl][PLAYER].pos();
        let player_action = handle_keys(tcod, game, all_objects);
        if player_action == PlayerAction::Exit {
            save_game(game, all_objects, start_objects).unwrap();
            break;
        } else if player_action == PlayerAction::TookTurn {
            game.turn += 1;
        }

        // let npcs take their turn
        if player_action == PlayerAction::TookTurn && all_objects[lvl][PLAYER].alive {
            for id in 0..all_objects[lvl].len() {
                if id != PLAYER && all_objects[lvl][id].ai.is_some() {
                    ai_take_turn(id, tcod, game, &mut all_objects[lvl]);
                }
            }
        }

        last_action = player_action;
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
                let (mut game, mut objects, mut start_objects) = new_game(tcod);
                play_game(tcod, &mut game, &mut objects, &mut start_objects);
            }
            Some(1) => {
                // load game
                match load_game() {
                    Ok((mut game, mut objects, mut start_objects)) => {
                        if !load_version_equals(tcod, &game) {
                            continue;
                        } else {
                            initialise_fov(tcod, &game.map());
                            play_game(tcod, &mut game, &mut objects, &mut start_objects);
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
    // Do some validation on data
    assert!(NUM_LVLS % 2 == 1);

    // set the FPS
    tcod::system::set_fps(LIMIT_FPS);

    // initialize the TCOD "Root" object
    let root: Root = Root::initializer()
        .font(FONT_IMG, FontLayout::AsciiInRow)
        .font_type(FontType::Default)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title(GAME_TITLE)
        .init();

    // use the TCOD "Root" object to create a mutable TCOD struct
    let mut tcod0 = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        key: Default::default(),
        mouse: Default::default(),
    };

    main_menu(&mut tcod0);
}
