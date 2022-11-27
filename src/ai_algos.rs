/*
  A central place for AI action algorithms
 */
// Import Third-Party
use rand::Rng;
use tcod::colors::*;

use crate::constants::PLAYER;
use crate::menus::Tcod;
use crate::moves::move_by;
use crate::moves::move_towards;
use crate::objects::Ai;
use crate::objects::Game;
use crate::objects::Object;
use crate::utils::mut_two;

// TODO: This logic could support movement speed.
// TODO: We should support "patroling" NPCs, that move even when the Player isn't around.


// Dispatch method to choose an AI algo to move the NPC
pub fn ai_take_turn(npc_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) {
    use Ai::*;
    if let Some(ai) = objects[npc_id].ai.take() {
        let new_ai = match ai {
            Basic => ai_basic(npc_id, tcod, game, objects),
            Confused {
                previous_ai,
                num_turns,
            } => ai_confused(npc_id, tcod, game, objects, previous_ai, num_turns),
        };
        objects[npc_id].ai = Some(new_ai);
    }
}


pub fn ai_basic(npc_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) -> Ai {
    // a basic npc takes its turn. If you can see it, it can see you
    let (npc_x, npc_y) = objects[npc_id].pos();
    if tcod.fov.is_in_fov(npc_x, npc_y) {
        if objects[npc_id].distance_to(&objects[PLAYER]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(npc_id, player_x, player_y, &game.map, objects);
        } else if objects[PLAYER].fighter.as_ref().map_or(false, |f| f.hp > 0) {
            // close enough, attack! (if the player is still alive.)
            let (npc, player) = mut_two(npc_id, PLAYER, objects);
            npc.attack(player, game);
        }
    }
    Ai::Basic
}


pub fn ai_confused(
    npc_id: usize,
    _tcod: &Tcod,
    game: &mut Game,
    objects: &mut [Object],
    previous_ai: Box<Ai>,
    num_turns: i32,
) -> Ai {
    if num_turns >= 0 {
        // still confused ...
        // move in a random direction, and decrease the number of turns confused
        move_by(
            npc_id,
            rand::thread_rng().gen_range(-1, 2),
            rand::thread_rng().gen_range(-1, 2),
            &game.map,
            objects,
        );
        Ai::Confused {
            previous_ai: previous_ai,
            num_turns: num_turns - 1,
        }
    } else {
        // restore the previous AI (this one will be deleted)
        game.messages.add(
            format!("The {} is no longer confused!", objects[npc_id].name),
            RED,
        );
        *previous_ai
    }
}
