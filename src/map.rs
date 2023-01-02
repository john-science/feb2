/*
  Highest-level map tooling
 */
// Import Third-Party
use serde::{Deserialize, Serialize};

// Import Locally
use crate::objects::Object;
use crate::map_algos::simple_fast::make_map_simple_fast;


// A tile of the map and its properties
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub blocked: bool,
    pub explored: bool,
    pub block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }
}


pub type Map = Vec<Vec<Tile>>;


// Useful testing function, when designing new map algorithms.
#[allow(dead_code)]
pub fn print_map(map: &Map) {
    map.iter().for_each(|row| {
        row.iter().for_each(|cell| {
            if cell.blocked { print!("#"); }
            else { print!(" "); }
        });
        print!("\n");
    });
    println!("\n");
}

pub fn make_map(all_objects: &mut Vec<Vec<Object>>, level: usize) -> (Map, (i32, i32), (i32, i32)) {
    return make_map_simple_fast(all_objects, level);
}
