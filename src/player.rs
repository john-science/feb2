/*
 Tools for dealing with Player logic
 */
use tcod::colors::WHITE;
use tcod::colors::YELLOW;

use crate::constants::CHARACTER_SCREEN_WIDTH;
use crate::constants::LEVEL_SCREEN_WIDTH;
use crate::constants::LEVEL_UP_BASE;
use crate::constants::LEVEL_UP_FACTOR;
use crate::constants::PLAYER;
use crate::menus::menu;
use crate::menus::msgbox;
use crate::menus::Tcod;
use crate::objects::Game;
use crate::objects::Object;


pub fn xp_to_level_up(lvl: i32) -> i32 {
    return LEVEL_UP_BASE + (lvl as i32 + 1) * LEVEL_UP_FACTOR;
}


pub fn level_up(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    let player = &mut objects[PLAYER];
    let level_up_xp = xp_to_level_up(player.level);
    // see if the player's experience is enough to level-up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        player.level += 1;
        game.messages.add(
            format!(
                "You grow stronger! You reached level {}!",
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


// TODO: Add inventory and anything else players are proud of.
pub fn character_screen(tcod: &mut Tcod, player: &Object, game: &Game) {
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
Defense: {}

Day: {}
Turn: {}
",
        fighter.karma,
        level,
        fighter.xp,
        level_up_xp,
        fighter.max_hp(),
        fighter.power(),
        fighter.defense(),
        game.day,
        game.turn,
    );
    msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
    }
}


pub fn reincarnate_reset(player: &mut Object) {
    player.alive = true;
    player.chr = '@';
    player.color = WHITE;
    player.level = 0;
    player.x = 0;
    player.y = 0;
    if let Some(fighter) = player.fighter.as_mut() {
        fighter.hp = 100;
        fighter.base_max_hp = 100;
        fighter.base_defense = 2;
        fighter.base_power = 3;
        fighter.xp = 0;
        fighter.inventory = vec![];
    }
}

