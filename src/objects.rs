/*
  Crucial Data Structures

  * Object
  * Game
  * Item, Slot, Equipment, Fighter
 */
use serde::{Deserialize, Serialize};
use tcod::colors::*;
use tcod::console::*;

use crate::constants::NUM_LVLS;
use crate::map::Map;
use crate::map::make_map;
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
    Chest,
    Head,
    Hand,
    Ring,
}


pub fn num_in_slot(slot: Slot) -> Option<usize> {
    match slot {
        Slot::Chest => Some(1),
        Slot::Head => Some(1),
        Slot::Hand => Some(2),
        Slot::Ring => Some(8),
    }
}


impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Slot::Chest => write!(f, "chest"),
            Slot::Head => write!(f, "head"),
            Slot::Hand => write!(f, "hand"),
            Slot::Ring => write!(f, "ring"),
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
    game.messages.add("You died. But you were already dead. This is not the end.", RED);

    // for added effect, temporarily transform the player into a corpse
    player.alive = false;
    player.chr = '%';
    player.color = DARK_RED;
}


// TODO: This doesn't handle one npc killing another.
fn npc_death(npc: &mut Object, game: &mut Game) {
    // transform it into a corpse! it doesn't block,
    // can't be attacked and doesn't move
    game.messages.add(
        format!(
            "{} is dead! (+{}XP / -{}K)",
            npc.name,
            npc.fighter.as_ref().unwrap().xp,
            Fighter::kill_karma(npc.fighter.as_ref().unwrap().xp, game.lvl as i32)
        ),
        ORANGE,
    );
    npc.alive = false;
    npc.chr = '%';
    npc.color = DARK_RED;
    npc.blocks = false;
    npc.fighter = None;
    npc.ai = None;
    npc.name = format!("remains of {}", npc.name);
}


// combat-related properties and methods (player or NPC)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub hp: i32,
    pub base_max_hp: u32,
    pub base_defense: i32,
    pub base_power: i32,
    pub xp: i32,
    pub karma: i32,
    pub on_death: DeathCallback,
    pub inventory: Vec<Object>,
}


impl Fighter {
    pub fn new(hp: i32, base_defense: i32, base_power: i32, xp: i32, is_npc: bool) -> Self {
        let on_death: DeathCallback = if is_npc {
            DeathCallback::Npc
        } else {
            DeathCallback::Player
        };

        Fighter {
            base_max_hp: hp as u32,
            hp: hp,
            base_defense: base_defense,
            base_power: base_power,
            xp: xp,
            karma: -1000,
            on_death: on_death,
            inventory: vec![],
        }
    }

    // heal by the given amount, without going over the maximum
    pub fn heal(&mut self, amount: i32) {
        let max_hp = self.max_hp();
        self.hp += amount;
        if self.hp > max_hp {
            self.hp = max_hp;
        }
    }


    pub fn power(&self) -> i32 {
        let bonus: i32 = self
            .get_all_equipped()
            .iter()
            .map(|e| e.power_bonus)
            .sum();
        self.base_power + bonus
    }

    pub fn defense(&self) -> i32 {
        let bonus: i32 = self
            .get_all_equipped()
            .iter()
            .map(|e| e.defense_bonus)
            .sum();
        self.base_defense + bonus
    }

    pub fn max_hp(&self) -> i32 {
        let bonus: i32 = self
            .get_all_equipped()
            .iter()
            .map(|e| e.max_hp_bonus)
            .sum();
        (self.base_max_hp as i32) + bonus
    }

    // returns a list of equipped items
    pub fn get_all_equipped(&self) -> Vec<Equipment> {
        self.inventory
            .iter()
            .filter(|item| item.equipment.map_or(false, |e| e.equipped))
            .map(|item| item.equipment.unwrap())
            .collect()
    }

    pub fn kill_rewards(&mut self, xp: i32, game_level: i32) {
        self.xp += xp;
        self.karma += Fighter::kill_karma(xp, game_level);
    }

    pub fn kill_karma(xp: i32, game_level: i32) -> i32 {
        return -xp * (game_level + 1);
    }
}


// This is a generic object: the player, a npc, an item, the stairs...
// It's represented by a character on screen (unless it's in an inventory).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub charges: i32,
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
            level: 1,
            charges: 1,
        }
    }

    pub fn get_color(&self) -> Color {
        if let Some(fighter) = self.fighter.as_ref() {
            let frac: f32 = fighter.hp as f32 / fighter.base_max_hp as f32;
            if frac >= 0.95 { return WHITE; }
            else {
                // player / NPC health displayed by color
                return Color{r: 255 - (255.0 * frac) as u8, g: (255.0 * frac) as u8, b: 0};
            }
        } else { return self.color; }
    }

    // set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.get_color());
        con.put_char(self.x, self.y, self.chr, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn is_stackable(&self) -> bool {
        return self.item.is_some() && self.equipment.is_none();
    }

    // return the distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        return ((dx.pow(2) + dy.pow(2)) as f32).sqrt();
    }

    pub fn take_damage(&mut self, damage: i32, game: &mut Game) -> i32 {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        // check for death, call the death function
        let mut xp: i32 = 0;
        if let Some(fighter) = self.fighter.as_mut() {
            if fighter.hp <= 0 {
                xp = fighter.xp;
                fighter.on_death.callback(self, game);
            }
        }

        return xp;
    }

    pub fn melee_attack(&mut self, target: &mut Object, game: &mut Game) {
        // a simple formula for attack damage
        let damage = self.fighter.as_ref().unwrap().power() - target.fighter.as_ref().unwrap().defense();
        if damage > 0 {
            // make the target take some damage
            game.messages.add(
                format!(
                    "{} attacks {} for {} hit points.",
                    self.name, target.name, damage
                ),
                WHITE
            );

            // bonus karma loss for damage done
            self.fighter.as_mut().unwrap().karma -= (damage as f64).sqrt() as i32;

            let xp = target.take_damage(damage, game);
            if xp > 0 {
                // yield experience to the player
                self.fighter.as_mut().unwrap().kill_rewards(xp, game.lvl as i32);
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

    // return the distance to some coordinates
    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

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
}


#[derive(Serialize, Deserialize)]
pub struct Game {
    pub maps: Vec<Map>,
    pub up_stairs: Vec<(i32, i32)>,
    pub down_stairs: Vec<(i32, i32)>,
    pub lvl: usize,
    pub messages: Messages,
    pub version: String,
    pub day: u32,
    pub turn: u32,
}

impl Game {
    pub fn new(objects: &mut Vec<Vec<Object>>) -> Self {
        let (m, up, down) = Game::make_maps(objects);
        Game {
            maps: m,
            up_stairs: up,
            down_stairs: down,
            lvl: 0,
            messages: Messages::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            day: 1,
            turn: 1,
        }
    }

    pub fn map(&mut self) -> &mut Map {
        return &mut self.maps[self.lvl];
    }

    fn make_maps(objects: &mut Vec<Vec<Object>>) -> (Vec<Map>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let mut maps: Vec<Map> = vec![];
        let mut up_stairs: Vec<(i32, i32)> = vec![];
        let mut down_stairs: Vec<(i32, i32)> = vec![];
        for i in 0..NUM_LVLS {
            let (m, up, down) = make_map(objects, i as usize);
            maps.push(m);
            up_stairs.push(up);
            down_stairs.push(down);
        }
        return (maps, up_stairs, down_stairs);
    }
}
