/*
  Playing around with how to reorganize the code.
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
