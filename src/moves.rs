/*
  Tools used to move the Player and NPCs
 */
// Import Third-Party
use crate::constants::PLAYER;
use crate::map::Map;
use crate::objects::Game;
use crate::objects::Object;
use crate::utils::mut_two;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    // first test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }

    // now check for any blocking objects
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}


// move by the given amount, if the destination is not blocked
pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) -> bool{
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
        return true;
    }
    return false;
}


pub fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, objects);
}


pub fn player_move_or_attack(dx: i32, dy: i32, game: &mut Game, objects: &mut [Object]) -> PlayerAction{
    use PlayerAction::*;

    // the coordinates the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = objects
        .iter()
        .position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.melee_attack(target, game);
            // TRYING to attack reduces karma
            player.fighter.as_mut().unwrap().karma -= 1;
            return TookTurn;
        }
        None => {
            if move_by(PLAYER, dx, dy, &game.map, objects) {
                return TookTurn;
            } else {
                return DidntTakeTurn;
            }
        }
    };
}
