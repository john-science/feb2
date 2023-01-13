/*
  Binary Space Partitiion (BSP) map algo
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
pub const ROOM_MIN_SIZE: i32 = 4;
pub const ITERATIONS: i32 = 6;


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

        // TODO: Also don't place the NPC if it is in FOV of the player
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


/**
 * Split a single cell into two (if possible).
 * Return corners of two new cells.
 * 
 * NOTE: If split not possible, return original corners,
 *       plus one set of dummies (negatives).
 * NOTE: The corner positions listed are inclusive.
 */
fn split_single_cell(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> ((i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let cell_width: i32 = max_x - min_x;
    let cell_height: i32 = max_y - min_y;

    if cell_width <= 2 * ROOM_MIN_SIZE && cell_height <= 2 * ROOM_MIN_SIZE {
        // Case 0: We can't go smaller, return this cell
        return ((min_x, min_y, max_x, max_y), (-1, -1, -1, -1))
    }

    let min_split_x: i32 = min_x + ROOM_MIN_SIZE;
    let max_split_x: i32 = max_x - ROOM_MIN_SIZE;
    let min_split_y: i32 = min_y + ROOM_MIN_SIZE;
    let max_split_y: i32 = max_y - ROOM_MIN_SIZE;

    let mut split_vert: bool = true;
    if cell_width <= 2 * ROOM_MIN_SIZE || (cell_height as f32 / cell_width as f32) > 3.0 {
        // Case 1: Split Horizontally
        split_vert = false;
    } else if cell_height <= 2 * ROOM_MIN_SIZE || (cell_width as f32 / cell_height as f32) > 3.0 {
        // Case 2: Split Vertically
        split_vert = true;
    } else {
        // Case 3: Split Vertically/Horizontally at random
        if rand::thread_rng().gen_range(0, 2) == 1 {
            split_vert = false;
        }
    }

    // return the 2 new cells
    if split_vert {
        let x: i32 = rand::thread_rng().gen_range(min_split_x, max_split_x + 1);
        return ((min_x, min_y, x, max_y), (x + 1, min_y, max_x, max_y))
    } else {
        let y: i32 = rand::thread_rng().gen_range(min_split_y, max_split_y + 1);
        return ((min_x, min_y, max_x, y), (min_x, y + 1, max_x, max_y))
    }
}


/**
 * Purely spatial part of the BSP
 * 
 * 1. Grab the space and divide it in 2. Save off the 2 new spaces
 * 2. Repeat, storing off the smaller spaces, N times.
 * 3. There are rules for (1) and (2). Minimum size rules.
 * 4. If a space is too small, just don't split it.
 */
fn binary_space_partition(width: i32, height: i32, iterations: i32) -> Vec<(i32, i32, i32, i32)> {
    // quick validation
    assert!(width > ROOM_MIN_SIZE);
    assert!(height > ROOM_MIN_SIZE);
    assert!(iterations > 0);

    // init the entire space as a cell
    let mut cells: Vec<(i32, i32, i32, i32)> = vec![];
    cells.push((0, 0, width, height));

    for _iter in 0..iterations {
        let mut new_cells: Vec<(i32, i32, i32, i32)> = vec![];

        // Go through each current cell and try to split it
        for (min_x, min_y, max_x, max_y) in cells.iter() {
            let (t1, t2) = split_single_cell(*min_x, *min_y, *max_x, *max_y);
            new_cells.push(t1);
            if t2.0 >=0 {
                // if the second tuple is all -1s, its not real data
                new_cells.push(t2);
            }
        }

        // wipe the old cells, and put in the new (smaller) ones
        cells.clear();
        for c in new_cells.iter() {
            cells.push(c.clone());
        }
    }

    return cells;
}


/**
 * Binary Space Partition for Map Generation
 * 
 * Step 1: Split space into pieces
 * Step 2: Add rooms
 * Step 3: Add hallways
 * Step 4: Add NPCs/Objects/Stairs into rooms
 */
pub fn bsp(all_objects: &mut Vec<Vec<Object>>, level: usize) -> (Map, (i32, i32), (i32, i32)) {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut down_posi: (i32, i32) = (-1, -1);
    let objects = &mut all_objects[level];

    // Divide the space up using BSP
    let parts: Vec<(i32, i32, i32, i32)> = binary_space_partition(MAP_WIDTH - 2, MAP_HEIGHT - 2, ITERATIONS);

    // generate a random set of roooms
    let mut rooms: Vec<Rect> = vec![];

    for (part_x0, part_y0, part_xf, part_yf) in parts.iter() {
        let part_width: i32 = part_xf - part_x0 + 1;
        let part_height: i32 = part_yf - part_y0 + 1;

        // random width and height
        let w = rand::thread_rng().gen_range((ROOM_MIN_SIZE + part_width) / 2, part_width + 1);
        let h = rand::thread_rng().gen_range((ROOM_MIN_SIZE + part_height) / 2, part_height + 1);
        // random position without going out of the boundaries of the map
        let x: i32 = part_x0 + (part_width - w) / 2;
        let y: i32 = part_y0 + (part_height - h) / 2;

        let new_room = Rect::new(x, y, w, h);

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
            let (prev_x, prev_y): (i32, i32) = rooms[rooms.len() - 1].center();

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

    // create up stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut up_stairs = Object::new(last_room_x, last_room_y, '>', "up-stairs", WHITE, false);
    up_stairs.always_visible = true;
    objects.push(up_stairs);

    return (map, (last_room_x, last_room_y), down_posi);
}
