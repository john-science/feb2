/*
  Rendering the User Interface
 */
// Import Std Libs
use std::collections::HashMap;

// Import Third-Party
use tcod::colors::*;
use tcod::console::*;
use tcod::input::{Mouse};
use tcod::map::{FovAlgorithm, Map as FovMap};

// Import Locally
use crate::constants::*;  // TODO: Will the complier make this more efficient for me?
use crate::menus::render_bar;
use crate::menus::Tcod;
use crate::objects::Fighter;
use crate::objects::Game;
use crate::objects::Object;
use crate::player::xp_to_level_up;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;


// return a string with the names of all objects under the mouse
fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    // find duplicate items, if any
    let mut name_map: HashMap<&String, i32> = HashMap::new();
    for nomen in names.iter() {
        if name_map.contains_key(nomen) {
            let count = *name_map.get(nomen).unwrap();
            name_map.insert(nomen, count + 1);
        } else {
            name_map.insert(nomen, 1);
        }
    }

    // pretty-print names like: healing potion x2
    let mut dedup: String = String::new();
    for (nomen, count) in name_map.iter() {
        if *count == 1 {
            dedup.push_str(nomen);
        }
        else if *count > 1 {
            dedup.push_str(&format!("{} x{}", nomen , count).to_string());
        }
        dedup.push_str(", ");
    }

    // remove trailing ", "
    dedup.pop();
    dedup.pop();
    dedup
}


pub fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player: &Object = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, true, FOV_ALGO);
    }

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible: bool = tcod.fov.is_in_fov(x, y);
            let wall: bool = game.map[x as usize][y as usize].block_sight;
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

    // show the player's HP
    let player_fighter: &Fighter = objects[PLAYER].fighter.as_ref().unwrap();
    let hp: i32 = player_fighter.hp;
    let max_hp: i32 = player_fighter.max_hp();
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

    // show the player's XP
    let xp = player_fighter.xp;
    let level_up_xp = xp_to_level_up(objects[PLAYER].level);
    render_bar(
        &mut tcod.panel,
        1,
        2,
        BAR_WIDTH,
        "XP",
        xp,
        level_up_xp,
        LIGHT_GREY,
        DARKER_GREY,
    );

    // show the player's karma
    let karma = player_fighter.karma;
    tcod.panel.print_ex(
        1,
        3,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Karma: {}", karma),
    );

    tcod.panel.print_ex(
        1,
        5,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Lvl {}: {}", game.map_level, LVL_NAMES[(game.map_level - 1) as usize]),
    );

    // TODO: panic scrolls if string is too long.
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
