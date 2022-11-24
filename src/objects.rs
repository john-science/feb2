/*
  Crucial Data Structures

  * Object
  * Game
  * Item, Slot, Equipment, Fighter
 */
use serde::{Deserialize, Serialize};
use tcod::colors::*;
use tcod::console::*;

use crate::map::Map;
use crate::menus::Messages;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    HealPot,
    LightningScroll,
    ConfuseScroll,
    FireballScroll,
    Sword,
    Shield,
}


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Slot {
    LeftHand,
    RightHand,
    Head,
}


impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Slot::LeftHand => write!(f, "left hand"),
            Slot::RightHand => write!(f, "right hand"),
            Slot::Head => write!(f, "head"),
        }
    }
}


pub enum UseResult {
    UsedUp,
    UsedAndKept,
    Cancelled,
}


// An object that can be equipped, yielding bonuses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    pub slot: Slot,
    pub equipped: bool,
    pub max_hp_bonus: i32,
    pub defense_bonus: i32,
    pub power_bonus: i32,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32,
    },
}


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Npc,
}


impl DeathCallback {
    pub fn callback(self, object: &mut Object, game: &mut Game) {
        use DeathCallback::*;
        let callback = match self {
            Player => player_death,
            Npc => npc_death,
        };
        callback(object, game);
    }
}


fn player_death(player: &mut Object, game: &mut Game) {
    // the game ended!
    game.messages.add("You died!", RED);

    // for added effect, transform the player into a corpse!
    player.chr = '%';
    player.color = DARK_RED;
}


// TODO: This doesn't handle one npc killing another.
fn npc_death(npc: &mut Object, game: &mut Game) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    game.messages.add(
        format!(
            "{} is dead! You gain {} experience points.",  // TODO: It's not death.
            npc.name,
            npc.fighter.unwrap().xp
        ),
        ORANGE,
    );
    npc.chr = '%';
    npc.color = DARK_RED;
    npc.blocks = false;
    npc.fighter = None;
    npc.ai = None;
    npc.name = format!("remains of {}", npc.name);
}


// combat-related properties and methods (player or NPC)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub hp: i32,  // TODO: u32?
    pub base_max_hp: i32,  // TODO: u32?
    pub base_defense: i32,
    pub base_power: i32,
    pub xp: i32,  // TODO: u32?
    pub on_death: DeathCallback,
}


// This is a generic object: the player, a npc, an item, the stairs...
// It's represented by a character on screen (unless it's in an inventory).
#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub chr: char,
    pub color: Color,
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub item: Option<Item>,
    pub equipment: Option<Equipment>,
    pub always_visible: bool,
    pub level: i32,
}

impl Object {
    pub fn new(x: i32, y: i32, chr: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            chr: chr,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
            fighter: None,
            ai: None,
            item: None,
            equipment: None,
            always_visible: false,
            level: 1,  // TODO: Just start counting at zero?
        }
    }

    // set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.chr, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    // return the distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        return ((dx.pow(2) + dy.pow(2)) as f32).sqrt();
    }

    // TODO: What is the benefit of returning Option over i32?  Just return 0 xp, right?
    pub fn take_damage(&mut self, damage: i32, game: &mut Game) -> Option<i32> {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        // check for death, call the death function
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
                fighter.on_death.callback(self, game);
                return Some(fighter.xp);
            }
        }
        return None;
    }

    pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
        // a simple formula for attack damage
        let damage = self.power(game) - target.defense(game);
        if damage > 0 {
            // make the target take some damage
            game.messages.add(
                format!(
                    "{} attacks {} for {} hit points.",
                    self.name, target.name, damage
                ),
                WHITE
            );

            if let Some(xp) = target.take_damage(damage, game) {
                // yield experience to the player
                self.fighter.as_mut().unwrap().xp += xp;
            }
        } else {
            game.messages.add(
                format!(
                    "{} attacks {} but it has no effect!",
                    self.name, target.name
                ),
                WHITE
            );
        }
    }

    // TODO: This seems like a silly place for this? Everything is on Object.
    // heal by the given amount, without going over the maximum
    pub fn heal(&mut self, amount: i32, game: &Game) {
        let max_hp = self.max_hp(game);
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > max_hp {
                fighter.hp = max_hp;
            }
        }
    }

    // return the distance to some coordinates
    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

    // TODO: NPCs should be able to have equipment.
    // Player equips an object and gets a message
    pub fn equip(&mut self, messages: &mut Messages) {
        if self.item.is_none() {
            messages.add(
                format!("Can't equip {:?} because it's not an Item.", self),
                RED,
            );
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if !equipment.equipped {
                equipment.equipped = true;
                messages.add(
                    format!("Equipped {} on {}.", self.name, equipment.slot),
                    LIGHT_GREEN,
                );
            }
        } else {
            messages.add(
                format!("Can't equip {:?} because it's not an Equipment.", self),
                RED,
            );
        }
    }

    // Player dequips an object and gets a message
    pub fn dequip(&mut self, messages: &mut Messages) {
        if self.item.is_none() {
            messages.add(
                format!("Can't dequip {:?} because it's not an Item.", self),
                RED,
            );
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if equipment.equipped {
                equipment.equipped = false;
                messages.add(
                    format!("Dequipped {} from {}.", self.name, equipment.slot),
                    LIGHT_YELLOW,
                );
            }
        } else {
            messages.add(
                format!("Can't dequip {:?} because it's not an Equipment.", self),
                RED,
            );
        }
    }

    pub fn power(&self, game: &Game) -> i32 {
        let base_power = self.fighter.map_or(0, |f| f.base_power);
        let bonus: i32 = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.power_bonus)
            .sum();
        base_power + bonus
    }

    pub fn defense(&self, game: &Game) -> i32 {
        let base_defense = self.fighter.map_or(0, |f| f.base_defense);
        let bonus: i32 = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.defense_bonus)
            .sum();
        base_defense + bonus
    }

    pub fn max_hp(&self, game: &Game) -> i32 {
        let base_max_hp = self.fighter.map_or(0, |f| f.base_max_hp);
        let bonus: i32 = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.max_hp_bonus)
            .sum();
        base_max_hp + bonus
    }

    // returns a list of equipped items
    pub fn get_all_equipped(&self, game: &Game) -> Vec<Equipment> {
        if self.name == "player" {  // TODO: Hacky. Prevents NPCs from having Inventories.
            game.inventory
                .iter()
                .filter(|item| item.equipment.map_or(false, |e| e.equipped))
                .map(|item| item.equipment.unwrap())
                .collect()
        } else {
            vec![] // other objects have no equipment
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Game {
    pub map: Map,
    pub messages: Messages,  // TODO: The entire history is saved, but it's not scrollable.
    pub inventory: Vec<Object>,
    pub map_level: u32,
}
