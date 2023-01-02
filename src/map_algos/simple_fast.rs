/*
  The simplest (and fastest) map-gen algo.
 */
// Import Std Libs
use std::cmp;

// Import Third-Party
use rand::Rng;
use tcod::colors::*;

// Import Locally
use crate::constants::MAP_WIDTH;
use crate::constants::MAP_HEIGHT;
use crate::constants::PLAYER;
use crate::loot_table::generate_floor_item;
use crate::map::Map;
use crate::map::Tile;
use crate::moves::is_blocked;
use crate::npc_table::generate_npc;
use crate::objects::Object;
use crate::transition::from_map_level;
use crate::transition::Transition;


// parameters for map generator
pub const ROOM_MAX_SIZE: i32 = 12;
pub const ROOM_MIN_SIZE: i32 = 6;
pub const MAX_ROOMS: i32 = 32;


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
            Transition { level: 0, value: 2 },
            Transition { level: 7, value: 3 },
            Transition { level: 14, value: 4 },
            Transition { level: 20, value: 6 },
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
            Transition { level: 0, value: 1 },
            Transition { level: 7, value: 2 },
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


pub fn make_map_simple_fast(all_objects: &mut Vec<Vec<Object>>, level: usize) -> (Map, (i32, i32), (i32, i32)) {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut down_posi: (i32, i32) = (-1, -1);
    let objects = &mut all_objects[level];

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

        // this means there are no intersections, so this room is valid
        if !failed {
            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                down_posi = (new_x, new_y);
                if level == 0 {
                    // this is the first room, where the player starts at
                    objects[PLAYER].set_pos(new_x, new_y);
                    down_posi = (new_x, new_y);
                } else {
                    let mut down_stairs = Object::new(new_x, new_y, '<', "down-stairs", WHITE, false);
                    down_stairs.always_visible = true;
                    objects.push(down_stairs);
                }
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
                place_objects(new_room, &map, objects, level as u32);
            }

            // finally, append the new room to the list
            rooms.push(new_room);
        }
    }

    // create up stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut up_stairs = Object::new(last_room_x, last_room_y, '>', "up-stairs", WHITE, false);
    up_stairs.always_visible = true;
    objects.push(up_stairs);

    return (map, (last_room_x, last_room_y), down_posi);
}
