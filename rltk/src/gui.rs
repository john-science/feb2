use rltk::{ RGB, Rltk, Console };
use specs::prelude::*;
use super::{ MAPWIDTH, MAPHEIGHT };

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    let box_height: i32 = 6;
    ctx.draw_box(0,
                 MAPHEIGHT,
                 MAPWIDTH as i32 - 1,
                 box_height,
                 RGB::named(rltk::WHITE),
                 RGB::named(rltk::BLACK));
}

