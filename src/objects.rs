/*
  TODO
 */
use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Heal,  // TODO: HealPot
    Lightning,  // TODO: LightningScroll
    Confuse,  // TODO: ConfuseScroll
    Fireball,  // TODO: FireballScroll
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
