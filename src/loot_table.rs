/*
 Implementation of the Loot Table
 */
use rand::distributions::{IndependentSample, Weighted, WeightedChoice};
use tcod::colors::*;

use crate::objects::Equipment;
use crate::objects::Item;
use crate::objects::Object;
use crate::objects::Slot;


#[derive(Clone)]
struct Loot {
    pub symbol: char,
    pub name: String,
    pub color: Color,
    pub item_type: Item,
    pub slot: Slot,
    pub hp_bonus: i32,
    pub defense_bonus: i32,
    pub attack_bonus: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub weight: u32,
}

impl Loot {
    fn new(
        symbol: char,
        name: &str,
        color: Color,
        item_type: Item,
        slot: Slot,
        hp_bonus: i32,
        defense_bonus: i32,
        attack_bonus: i32,
        min_level: i32,
        max_level: i32,
        weight: u32,
    ) -> Self {
        assert!(min_level < max_level);

        Loot {
            symbol: symbol,
            name: name.to_string(),
            color: color,
            item_type: item_type,
            slot: slot,
            hp_bonus: hp_bonus,
            defense_bonus: defense_bonus,
            attack_bonus: attack_bonus,
            min_level: min_level,
            max_level: max_level,
            weight: weight,
        }
    }

    fn generate(&self) -> Object {
        // NOTE: Setting to an impossible location
        let mut obj = Object::new(-1, -1, self.symbol, &self.name, self.color, false);
        obj.item = Some(self.item_type);
        if (self.hp_bonus + self.defense_bonus + self.attack_bonus) > 0 {
            obj.equipment = Some(Equipment {
                equipped: false,
                slot: self.slot,
                max_hp_bonus: self.hp_bonus,
                defense_bonus: self.defense_bonus,
                power_bonus: self.attack_bonus,
            });
        }
        return obj;
    }
}

/* Symbology

 [ = armor/shields
 - = dagger/small melee weapon
 / = sword/large melee weapon
 | = two-handed weapons
 ! = potions
 ~ = scroll
 # = books
 = = rings
 " = amulets
 % = food / corpses
 $ = gold (if there ever is any)

*/
fn loot_table() -> Vec<Loot> {
    return vec![
        // weapons
        Loot::new('/', "sword breaker", DARK_BLUE, Item::Sword, Slot::Hand, 0, 2, 4, 18, 99, 10),
        Loot::new('/', "long sword", BLUE, Item::Sword, Slot::Hand, 0, 0, 5, 14, 99, 10),
        Loot::new('/', "sword", SKY, Item::Sword, Slot::Hand, 0, 0, 4, 7, 99, 15),
        Loot::new('/', "short sword", LIGHT_BLUE, Item::Sword, Slot::Hand, 0, 0, 3, 4, 14, 15),
        Loot::new('-', "dagger", SKY, Item::Sword, Slot::Hand, 0, 0, 2, -99, 4, 15),
        // shields
        Loot::new('[', "shieldwall", DARKER_ORANGE, Item::Shield, Slot::Hand, 0, 5, 0, 15, 99, 10),
        Loot::new('[', "shield", ORANGE, Item::Shield, Slot::Hand, 0, 3, 0, 11, 20, 15),
        Loot::new('[', "buckler", LIGHT_ORANGE, Item::Shield, Slot::Hand, 0, 2, 0, 8, 17, 25),
        // potions
        Loot::new('!', "healing potion", VIOLET, Item::HealPot, Slot::Head, 0, 0, 0, -99, 99, 35),
        // scrolls
        Loot::new('~', "scroll of lightning bolt", LIGHT_BLUE, Item::LightningScroll, Slot::Head, 0, 0, 0, 4, 99, 25),
        Loot::new('~', "scroll of fireball", RED, Item::FireballScroll, Slot::Head, 0, 0, 0, 2, 99, 25),
        Loot::new('~', "scroll of confusion", LIGHT_GREEN, Item::ConfuseScroll, Slot::Head, 0, 0, 0, 1, 99, 10),
    ];
}


fn trim_loot_by_level(level: i32) -> Vec<Loot> {
    return loot_table()
           .iter()
           .filter(|&row| level >= row.min_level && level <= row.max_level)
           .cloned()
           .collect::<Vec<Loot>>();
}


// TODO: Convert to generate multiple items
pub fn generate_floor_item(level: i32) -> Object {
    // find all items possible on a given floor, and their weights
    let table: Vec<Loot> = trim_loot_by_level(level);
    assert!(table.len() > 0);

    // do the weighted random chance thing
    let mut chances = vec![];
    for (i, row) in table.iter().enumerate() {
        chances.push(Weighted{weight: row.weight, item: i});
    }

    let choices = WeightedChoice::new(&mut chances);

    return table[choices.ind_sample(&mut rand::thread_rng())].generate();
}

