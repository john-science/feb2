/*
 Implementation of the NPC Table
 */
use rand::distributions::{IndependentSample, Weighted, WeightedChoice};
use tcod::colors::*;

use crate::objects::Ai;
use crate::objects::Fighter;
use crate::objects::Object;


#[derive(Clone)]
pub struct NPC {
    pub symbol: char,
    pub name: String,
    pub color: Color,
    pub ai: Ai,
    pub max_hp: i32,
    pub defense: i32,
    pub attack: i32,
    pub xp: i32,
    pub start_hp: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub weight: u32,
}

impl NPC {
    pub fn new(
        symbol: char,
        name: &str,
        color: Color,
        ai: Ai,
        max_hp: i32,
        defense: i32,
        attack: i32,
        xp: i32,
        start_hp: i32,
        min_level: i32,
        max_level: i32,
        weight: u32,
    ) -> Self {
            assert!(min_level < max_level);

            NPC {
            symbol: symbol,
            name: name.to_string(),
            color: color,
            ai: ai,
            max_hp: max_hp,
            defense: defense,
            attack: attack,
            xp: xp,
            start_hp: start_hp,
            min_level: min_level,
            max_level: max_level,
            weight: weight,
        }
    }

    pub fn generate(&self) -> Object {
        // NOTE: Setting to an impossible location
        let mut npc = Object::new(-1, -1, self.symbol, &self.name, self.color, true);
        npc.ai = Some(self.ai.clone());
        let mut fighter = Fighter::new(self.max_hp, self.defense, self.attack, self.xp, true);
        fighter.hp = self.start_hp;
        npc.fighter = Some(fighter);
        npc.alive = true;
        return npc;
    }
}


fn npc_table() -> Vec<NPC> {
    return vec![
        NPC::new('O', "orc", DESATURATED_GREEN, Ai::Basic, 20, 0, 4, 35, 20, -99, 99, 100),
        NPC::new('T', "troll", DARKER_GREEN, Ai::Basic, 60, 2, 8, 100, 30, 1, 99, 50),
    ];
}


fn trim_npcs_by_level(level: i32) -> Vec<NPC> {
    return npc_table()
           .iter()
           .filter(|&row| level >= row.min_level && level <= row.max_level)
           .cloned()
           .collect::<Vec<NPC>>();
}


// TODO: Convert to generate multiple NPCs
pub fn generate_npc(level: i32) -> Object {
    // find all NPCs possible on a given floor, and their weights
    let table: Vec<NPC> = trim_npcs_by_level(level);
    assert!(table.len() > 0);

    // do the weighted random chance thing
    let mut chances = vec![];
    for (i, row) in table.iter().enumerate() {
        chances.push(Weighted{weight: row.weight, item: i});
    }

    let choices = WeightedChoice::new(&mut chances);

    return table[choices.ind_sample(&mut rand::thread_rng())].generate();
}

