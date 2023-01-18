use specs::prelude::*;
use super::{Viewshed, Position, Map};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut vs, pos) = data;

        for (vs, pos) in (&mut vs, &pos).join() {
            vs.visible_tiles.clear();
            // TODO: &*map ?  Why???
            vs.visible_tiles = field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
            vs.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height );
        }
    }
}

