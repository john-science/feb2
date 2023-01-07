/*
  Modified Binary Space Partitiion (BSP) map algo

  Modificified:
  - to randomly skip the split
  - with custom logic for room shapes
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
pub const ROOM_MIN_SIZE: i32 = 5;
pub const ITERATIONS: i32 = 5;


// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub xf: i32,
    pub yf: i32,
}

impl Rect {
    pub fn new(x0: i32, y0: i32, xf: i32, yf: i32) -> Self {
        Rect {
            x0: x0,
            y0: y0,
            xf: xf,
            yf: yf,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        // find the center of the Rect
        let center_x: i32 = (self.x0 + self.xf) / 2;
        let center_y: i32 = (self.y0 + self.yf) / 2;
        return (center_x, center_y);
    }
}


fn carve_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x0 + 1)..room.xf {
        for y in (room.y0 + 1)..room.yf {
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


fn create_room(part: Rect, map: &mut Map) -> Rect {
    let part_width: i32 = part.xf - part.x0 + 1;
    let part_height: i32 = part.yf - part.y0 + 1;

    // random width and height
    let w = rand::thread_rng().gen_range((ROOM_MIN_SIZE + part_width) / 2, part_width + 1);
    let h = rand::thread_rng().gen_range((ROOM_MIN_SIZE + part_height) / 2, part_height + 1);
    // random position without going out of the boundaries of the map
    let x: i32 = part.x0 + (part_width - w) / 2;
    let y: i32 = part.y0 + (part_height - h) / 2;

    let new_room = Rect::new(x, y, x + w, y + h);

    // "paint" it to the map's tiles
    carve_room(new_room, map);

    return new_room;
}


// TODO: Transition this to taking the entire Partition
fn place_objects(part: Rect, map: &Map, objects: &mut Vec<Object>, level: u32) {
    // value is chance-in-1000 that an NPC will be in a cell
    let npc_chance: u32 = from_map_level(
        &[
            Transition { level: 0, value: 12 },
            Transition { level: 20, value: 100 },
        ],
        level,
    );

    // value is chance-in-1000 that an item will be in a cell
    let item_chance: u32 = from_map_level(
        &[
            Transition { level: 0, value: 12 },
            Transition { level: 10, value: 32 },
            Transition { level: 11, value: 12 },
            Transition { level: 20, value: 64 },
        ],
        level,
    );

    // loop through every cell in the partition and roll the dice to place an NPC or an item
    for x in part.x0..part.xf+1 {
        for y in part.y0..part.yf+1 {
            if !is_blocked(x, y, map, objects) {
                let chance: u32 = rand::thread_rng().gen_range(0, 1000) as u32;
                if chance < npc_chance {
                    let mut npc = generate_npc(level as i32);
                    npc.x = x;
                    npc.y = y;
                    objects.push(npc);
                }

                let chance: u32 = rand::thread_rng().gen_range(0, 1000) as u32;
                if chance < item_chance {
                    let mut item = generate_floor_item(level as i32);
                    item.x = x;
                    item.y = y;
                    objects.push(item);
                }
            }
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
fn split_single_cell(cell: Rect) -> (Rect, Rect) {
    let cell_width: i32 = cell.xf - cell.x0;
    let cell_height: i32 = cell.yf - cell.y0;

    if cell_width <= 2 * ROOM_MIN_SIZE && cell_height <= 2 * ROOM_MIN_SIZE {
        // Case 0: We can't go smaller, return this cell
        return (Rect::new(cell.x0, cell.y0, cell.xf, cell.yf),
                Rect::new(-1, -1, -1, -1))
    }

    let min_split_x: i32 = cell.x0 + ROOM_MIN_SIZE;
    let max_split_x: i32 = cell.xf - ROOM_MIN_SIZE;
    let min_split_y: i32 = cell.y0 + ROOM_MIN_SIZE;
    let max_split_y: i32 = cell.yf - ROOM_MIN_SIZE;

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
        return (Rect::new(cell.x0, cell.y0, x, cell.yf),
                Rect::new(x + 1, cell.y0, cell.xf, cell.yf))
    } else {
        let y: i32 = rand::thread_rng().gen_range(min_split_y, max_split_y + 1);
        return (Rect::new(cell.x0, cell.y0, cell.xf, y),
                Rect::new(cell.x0, y + 1, cell.xf, cell.yf))
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
fn binary_space_partition(width: i32, height: i32, iterations: i32) -> Vec<Rect> {
    // quick validation
    assert!(width > ROOM_MIN_SIZE);
    assert!(height > ROOM_MIN_SIZE);
    assert!(iterations > 0);

    // init the entire space as a cell
    let mut cells: Vec<Rect> = vec![];
    cells.push(Rect::new(0, 0, width, height));

    for iter in 0..iterations {
        let mut new_cells: Vec<Rect> = vec![];

        // Go through each current cell and try to split it
        for this_cell in cells.iter() {
            if iter > 1 && rand::thread_rng().gen_range(0, 21) == 0 {
                // random 1-in-20 chance to NOT split
                new_cells.push(*this_cell);
            } else {
                let (t1, t2) = split_single_cell(*this_cell);
                new_cells.push(t1);
                if t2.x0 >=0 {
                    // if the second tuple is all -1s, its not real data
                    new_cells.push(t2);
                }
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
pub fn bsp_mod(all_objects: &mut Vec<Vec<Object>>, level: usize) -> (Map, (i32, i32), (i32, i32)) {
    // fill map with "unblocked" tiles
    let mut map: Vec<Vec<Tile>> = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut down_posi: (i32, i32) = (-1, -1);
    let objects = &mut all_objects[level];

    // Divide the space up using BSP
    let parts: Vec<Rect> = binary_space_partition(MAP_WIDTH - 2, MAP_HEIGHT - 2, ITERATIONS);

    // generate a random set of roooms
    let mut rooms: Vec<Rect> = vec![];

    for part in parts.iter() {
        // create a room, using complicated, custom logic
        let new_room = create_room(*part, &mut map);

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
            place_objects(*part, &map, objects, level as u32);
        }

        // finally, append the new room to the list
        rooms.push(new_room);
    }

    // TODO: create a tunnel between two random, non-adjacent parts

    // create up stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut up_stairs = Object::new(last_room_x, last_room_y, '>', "up-stairs", WHITE, false);
    up_stairs.always_visible = true;
    objects.push(up_stairs);

    return (map, (last_room_x, last_room_y), down_posi);
}
