/*
  Map and Map-Creation Tools
 */
// Import Std Libs
use std::cmp;

// Import Third-Party
use rand::Rng;
use serde::{Deserialize, Serialize};
use tcod::colors::*;

// Import Locally
use crate::constants::MAP_WIDTH;
use crate::constants::MAP_HEIGHT;
use crate::constants::MAX_ROOMS;
use crate::constants::PLAYER;
use crate::constants::ROOM_MIN_SIZE;
use crate::constants::ROOM_MAX_SIZE;
use crate::loot_table::generate_floor_item;
use crate::moves::is_blocked;
use crate::npc_table::generate_npc;
use crate::objects::Object;
use crate::transition::from_map_level;
use crate::transition::Transition;


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


// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        // find the center of the Rect
        let center_x: i32 = (self.x1 + self.x2) / 2;
        let center_y: i32 = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}


fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}


fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>, level: u32) {
    // maximum number of npcs per room
    let max_npcs = from_map_level(
        &[
            Transition { level: 1, value: 2 },
            Transition { level: 4, value: 3 },
            Transition { level: 6, value: 5 },
        ],
        level,
    );

    // choose random number of npcs
    let num_npcs = rand::thread_rng().gen_range(0, max_npcs + 1);

    for _ in 0..num_npcs {
        // choose random spot for this npc
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // only place it if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            let mut npc = generate_npc(level as i32);
            npc.x = x;
            npc.y = y;
            objects.push(npc);
        }
    }

    // maximum number of items per room
    let max_items = from_map_level(
        &[
            Transition { level: 1, value: 1 },
            Transition { level: 4, value: 2 },
        ],
        level,
    );

    // choose random number of items
    let num_items = rand::thread_rng().gen_range(0, max_items + 1);

    for _ in 0..num_items {
        // choose random spot for this item
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // only place it if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            let mut item = generate_floor_item(level as i32);
            item.x = x;
            item.y = y;
            objects.push(item);
        }
    }
}


// TODO: Bug. I entered a game and there were 3 npcs in the start room. (No npcs should spawn in FOV of the player at the Start.)
pub fn make_map(objects: &mut Vec<Object>, level: u32) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // TODO: Player cannot visit old levels! OMG, this deletes everything!!!!!!
    // Player is the first element, remove everything else.
    // NOTE: works only when the player is the first object!
    assert_eq!(&objects[PLAYER] as *const Object, &objects[0] as *const Object);
    objects.truncate(1);

    // generate a random set of roooms
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }

                // add some content to this room, such as npcs
                place_objects(new_room, &map, objects, level);
            }

            // finally, append the new room to the list
            rooms.push(new_room);
        }
    }

    // create stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut stairs = Object::new(last_room_x, last_room_y, '>', "stairs", WHITE, false);
    stairs.always_visible = true;
    objects.push(stairs);

    return map;
}
