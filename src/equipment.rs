/*
  Equipment Tools:
  Using, (Un)Equiping, Drop/Pick up
 */
// Import Third-Party
use tcod::colors::*;

// Import Locally
use crate::constants::PLAYER;
use crate::magic::cast_confuse;
use crate::magic::cast_fireball;
use crate::magic::cast_heal;
use crate::magic::cast_lightning;
use crate::menus::Messages;
use crate::menus::Tcod;
use crate::objects::Fighter;
use crate::objects::Game;
use crate::objects::Item;
use crate::objects::Object;
use crate::objects::Slot;
use crate::objects::UseResult;


fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inv_id, item) in inventory.iter().enumerate() {
        if item
            .equipment
            .as_ref()
            .map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inv_id);
        }
    }
    return None;
}


fn toggle_equipment(inv_id: usize, _tcod: &mut Tcod, game: &mut Game, objs: &mut [Object]) -> UseResult {
    let fighter: &mut Fighter = objs[PLAYER].fighter.as_mut().unwrap();
    let equipment = match fighter.inventory[inv_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };
    if equipment.equipped {
        fighter.inventory[inv_id].dequip(&mut game.messages);
    } else {
        // if the slot is already being used, dequip whatever is there first
        if let Some(current) = get_equipped_in_slot(equipment.slot, &fighter.inventory) {
            fighter.inventory[current].dequip(&mut game.messages);
        }
        fighter.inventory[inv_id].equip(&mut game.messages);
    }
    UseResult::UsedAndKept
}



// TODO: Should be able to hold 52 things: a-z and A-Z (maybe 62 with 0-9 for gear?)
// TODO: Some items should stack, like scrolls. Maybe health pots.
// add to the player's inventory and remove from the map
pub fn pick_item_up(object_id: usize, picker_id: usize, messages: &mut Messages, objects: &mut Vec<Object>) {
    let is_player: bool = if picker_id == PLAYER { true } else { false };
    let fighter: &Fighter = objects[picker_id].fighter.as_ref().unwrap();
    if is_player && fighter.inventory.len() >= 26{
        messages.add(
            format!(
                "Your inventory is full, you cannot pick up {}.",
                objects[object_id].name
            ),
            RED,
        );
    } else {
        let item: Object = objects.swap_remove(object_id);
        let item_name: String = item.name.clone();
        objects[picker_id].fighter.as_mut().unwrap().inventory.push(item);
        messages.add(format!("{} picked up a {}.", objects[picker_id].name, item_name), GREEN);
    }
}


pub fn drop_item(inv_id: usize, dropper_id: usize, messages: &mut Messages, objects: &mut Vec<Object>) {
    let fighter: &mut Fighter = objects[dropper_id].fighter.as_mut().unwrap();
    let mut item: Object = fighter.inventory.remove(inv_id);
    if item.equipment.is_some() {
        item.dequip(messages);
    }
    item.set_pos(objects[dropper_id].x, objects[dropper_id].y);
    messages.add(format!("{} dropped a {}.", objects[dropper_id].name, item.name), YELLOW);
    objects.push(item);
}


// The player should also be able to use scrolls/potions they are standing on (and is useable).
pub fn player_use_item(inv_id: usize, tcod: &mut Tcod, game: &mut Game, objs: &mut Vec<Object>) {
    use Item::*;
    // just call the "use_function" if it is defined
    let fighter: &Fighter = objs[PLAYER].fighter.as_ref().unwrap();
    if let Some(item) = fighter.inventory[inv_id].item {
        let on_use = match item {
            // TODO: This seems like a limiting design.
            HealPot => cast_heal,
            LightningScroll => cast_lightning,
            ConfuseScroll => cast_confuse,
            FireballScroll => cast_fireball,
            Sword => toggle_equipment,
            Shield => toggle_equipment,
        };
        match on_use(inv_id, tcod, game, objs) {
            UseResult::UsedUp => {
                // destroy after use, unless it was cancelled for some reason
                objs[PLAYER].fighter.as_mut().unwrap().inventory.remove(inv_id);
            }
            UseResult::UsedAndKept => {} // do nothing
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!("The {} cannot be used.", fighter.inventory[inv_id].name),
            WHITE,
        );
    }
}

