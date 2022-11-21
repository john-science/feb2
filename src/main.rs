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
use tcod::input::{self, Event, Key, Mouse};
use tcod::map::{FovAlgorithm, Map as FovMap};

// Import Locally
mod ai_algos;
mod map;
mod menus;
mod moves;
mod objects;
mod transition;
mod utils;
use ai_algos::ai_take_turn;
use map::create_h_tunnel;
use map::create_room;
use map::create_v_tunnel;
use map::Map;
use map::Rect;
use map::Tile;
use menus::menu;
use menus::Messages;
use menus::msgbox;
use menus::render_bar;
use menus::SCREEN_HEIGHT;
use menus::SCREEN_WIDTH;
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
use objects::UseResult;
use transition::from_map_level;
use transition::Transition;

// sizes and coordinates relevant for the GUI
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
const CHARACTER_SCREEN_WIDTH: i32 = 30;

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;

// field-of-view
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

// colors for map objects
const COLOR_DARK_WALL: Color = Color { r: 6, g: 3, b: 1 };
const COLOR_DARK_GROUND: Color = Color { r: 81, g: 44, b: 15 };
const COLOR_LIGHT_WALL: Color = Color { r: 30, g: 16, b: 5 };
const COLOR_LIGHT_GROUND: Color = Color { r: 124, g: 65, b: 21 };

// parameters for map generator
const ROOM_MAX_SIZE: i32 = 12;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 32;
const INVENTORY_WIDTH: i32 = 50;

// TODO: Should depend on skills
const HEAL_AMOUNT: i32 = 40;
const LIGHTNING_DAMAGE: i32 = 40;
const LIGHTNING_RANGE: i32 = 5;
const CONFUSE_RANGE: i32 = 8;
const CONFUSE_NUM_TURNS: i32 = 10;
const FIREBALL_RADIUS: i32 = 3;
const FIREBALL_DAMAGE: i32 = 25;

// experience and level-ups (BASE + level * FACTOR)
const LEVEL_UP_BASE: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;
const LEVEL_SCREEN_WIDTH: i32 = 40;

// player will always be the first object
const PLAYER: usize = 0;


// TODO: Break this into multiple files.
// TODO: The color of potions, or maybe the font, is hard to read.
// TODO: I would like to have item/NPC/player data in data files that are ingested at compile time.


#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
        if item
            .equipment
            .as_ref()
            .map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inventory_id);
        }
    }
    return None;
}


fn toggle_equipment(
    inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    _objects: &mut [Object],
) -> UseResult {
    let equipment = match game.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };
    if equipment.equipped {
        game.inventory[inventory_id].dequip(&mut game.messages);
    } else {
        // if the slot is already being used, dequip whatever is there first
        if let Some(current) = get_equipped_in_slot(equipment.slot, &game.inventory) {
            game.inventory[current].dequip(&mut game.messages);
        }
        game.inventory[inventory_id].equip(&mut game.messages);
    }
    UseResult::UsedAndKept
}


// find closest enemy, up to a maximum range, and in the player's FOV
fn closest_npc(tcod: &Tcod, objects: &[Object], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32; // start with (slightly more than) maximum range

    for (id, object) in objects.iter().enumerate() {
        if (id != PLAYER)
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.x, object.y)
        {
            // calculate distance between this object and the player
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                // it's closer, so remember it
                closest_enemy = Some(id);
                closest_dist = dist;
            }
        }
    }
    closest_enemy
}


// NOTE: This expects only the player to cast lightning
fn cast_lightning(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    // find closest enemy (inside a maximum range and damage it)
    let npc_id = closest_npc(tcod, objects, LIGHTNING_RANGE);
    if let Some(npc_id) = npc_id {
        // zap it!
        game.messages.add(
            format!(
                "A lightning bolt strikes the {} with a loud thunder! \
                 The damage is {} hit points.",
                objects[npc_id].name, LIGHTNING_DAMAGE
            ),
            LIGHT_BLUE,
        );
        if let Some(xp) = objects[npc_id].take_damage(LIGHTNING_DAMAGE, game) {
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
        }
        return UseResult::UsedUp;
    } else {
        // no enemy found within maximum range
        game.messages.add("No enemy is close enough to strike.", RED);
        return UseResult::Cancelled;
    }
}


// NOTE: This expects only the player to cast fireball
fn cast_fireball(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // ask the player for a target tile to throw a fireball at
    game.messages.add(
        "Left-click a target tile for the fireball, or right-click to cancel.",
        LIGHT_CYAN,
    );
    let (x, y) = match target_tile(tcod, game, objects, None) {
        Some(tile_pos) => tile_pos,
        None => return UseResult::Cancelled,
    };
    game.messages.add(
        format!(
            "The fireball explodes, burning everything within {} tiles!",
            FIREBALL_RADIUS
        ),
        ORANGE,
    );

    let mut xp_to_gain = 0;
    for (id, obj) in objects.iter_mut().enumerate() {
        if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() {
            game.messages.add(
                format!(
                    "The {} gets burned for {} hit points.",
                    obj.name, FIREBALL_DAMAGE
                ),
                ORANGE,
            );
            if let Some(xp) = obj.take_damage(FIREBALL_DAMAGE, game) {
                if id != PLAYER {
                    // Don't reward the player for burning themself.
                    xp_to_gain += xp;
                }
            }
        }
    }
    objects[PLAYER].fighter.as_mut().unwrap().xp += xp_to_gain;

    UseResult::UsedUp
}


// return a string with the names of all objects under the mouse
fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // join the names, separated by commas
}


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



// TODO: Some items should stack, like health potions, and money.
// add to the player's inventory and remove from the map
fn pick_item_up(object_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if game.inventory.len() >= 26 {
        game.messages.add(
            format!(
                "Your inventory is full, you cannot pick up {}.",
                objects[object_id].name
            ),
            RED,
        );
    } else {
        let item = objects.swap_remove(object_id);
        game.messages.add(format!("You picked up a {}.", item.name), GREEN);
        game.inventory.push(item);
    }
}


fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    let mut item = game.inventory.remove(inventory_id);
    if item.equipment.is_some() {
        item.dequip(&mut game.messages);
    }
    item.set_pos(objects[PLAYER].x, objects[PLAYER].y);
    game.messages.add(format!("You dropped a {}.", item.name), YELLOW);
    objects.push(item);
}


// The player should also be able to use scrolls/potions they are standing on (and is useable).
fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    use Item::*;
    // just call the "use_function" if it is defined
    if let Some(item) = game.inventory[inventory_id].item {
        let on_use = match item {
            // TODO: This seems like a limiting design.
            Heal => cast_heal,
            Lightning => cast_lightning,
            Confuse => cast_confuse,
            Fireball => cast_fireball,
            Sword => toggle_equipment,
            Shield => toggle_equipment,
        };
        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                // destroy after use, unless it was cancelled for some reason
                game.inventory.remove(inventory_id);
            }
            UseResult::UsedAndKept => {} // do nothing
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!("The {} cannot be used.", game.inventory[inventory_id].name),
            WHITE,
        );
    }
}


// TODO: I'd like to be able to tab-select too. And auto-select closest.
// returns a clicked npc inside FOV up to a range, or None if right-clicked
fn target_npc(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<usize> {
    loop {
        match target_tile(tcod, game, objects, max_range) {
            Some((x, y)) => {
                // return the first clicked npc, otherwise continue looping
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None,
        }
    }
}



fn cast_heal(
    _inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // heal the player
    let player = &mut objects[PLAYER];
    if let Some(fighter) = player.fighter {
        if fighter.hp == player.max_hp(game) {
            game.messages.add("You are already at full health.", RED);
            return UseResult::Cancelled;
        }
        game.messages
            .add("Your wounds start to feel better!", LIGHT_VIOLET);
        player.heal(HEAL_AMOUNT, game);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}


fn cast_confuse(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    // ask the player for a target to confuse
    game.messages.add(
        "Left-click an enemy to confuse it, or right-click to cancel.",
        LIGHT_CYAN,
    );
    let npc_id = target_npc(tcod, game, objects, Some(CONFUSE_RANGE as f32));
    if let Some(npc_id) = npc_id {
        let old_ai = objects[npc_id].ai.take().unwrap_or(Ai::Basic);
        // replace the npc's AI with a "confused" one; after
        // some turns it will restore the old AI
        objects[npc_id].ai = Some(Ai::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        game.messages.add(
            format!(
                "The eyes of {} look vacant, as he starts to stumble around!",
                objects[npc_id].name
            ),
            LIGHT_GREEN,
        );
        UseResult::UsedUp
    } else {
        // no enemy fonud within maximum range
        game.messages.add("No enemy is close enough to strike.", RED);
        UseResult::Cancelled
    }
}


// Return the position of a tile left-clicked in player's FOV (optionally in a range),
// or (None, None) if right-clicked.
fn target_tile(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<(i32, i32)> {
    use tcod::input::KeyCode::Escape;
    loop {
        // render the screen. this erases the inventory and shows the names of
        // objects under the mouse.
        tcod.root.flush();
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => tcod.key = k,
            None => tcod.key = Default::default(),
        }
        render_all(tcod, game, objects, false);

        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        // accept the target if the player clicked in FOV, and in case a range
        // is specified, if it's in that range
        let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
        let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);
        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        if tcod.mouse.rbutton_pressed || tcod.key.code == Escape {
            return None; // cancel if the player right-clicked or pressed Escape
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


// TODO: Why does this return an Option, not an i32? Couldn't we just return 0, not None?
fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    // show a menu with each item of the inventory as an option
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory
            .iter()
            .map(|item| {
                // show additional information, in case it's equipped
                match item.equipment {
                    Some(equipment) if equipment.equipped => {
                        format!("{} (on {})", item.name, equipment.slot)
                    }
                    _ => item.name.clone(),
                }
            })
            .collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // if an item was chosen, return it
    if inventory.len() > 0 {
        return inventory_index;
    } else {
        return None;
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


fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                // outside of field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                // since it's visible, explore it
                *explored = true;
            }
            if *explored {
                // show explored tiles only (any visible tile is explored already)
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    // draw all objects in the list, in correct order
    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| {
            tcod.fov.is_in_fov(o.x, o.y)
                || (o.always_visible && game.map[o.x as usize][o.y as usize].explored)
        })
        .collect();
    // sort so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }

    // prepare to render the GUI panel
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    // show the player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].max_hp(game);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED,
    );

    tcod.panel.print_ex(
        1,
        3,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Level: {}", game.map_level),  // TODO: working... levels or?
    );

    // TODO: fails for multiple items on the same tile.
    // display names of objects under the mouse
    tcod.panel.set_default_foreground(LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, objects, &tcod.fov),
    );

    // print the game messages, one line at a time
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

    // blit the contents of `panel` to the root console
    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );
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
