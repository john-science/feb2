use rltk::{GameState, Point, Rltk, RGB, RltkBuilder};
use specs::prelude::*;

mod components;
mod map;
mod monster_ai_system;
mod player;
mod rect;
mod visibility_system;

pub use components::*;
pub use map::*;
pub use monster_ai_system::MonsterAI;
use player::*;
pub use rect::Rect;
use visibility_system::VisibilitySystem;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

pub struct State {
    ecs: World,
    pub runstate : RunState
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }
    }
}


fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("February Second")
        .build()?;

    let mut gs = State{
        ecs: World::new(),
        runstate : RunState::Running
    };
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();  // TODO: Is it possible in ECS to say there can be only one of these?
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(Point::new(player_x, player_y));

    // putting in some placeholder NPCs
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();

        let glyph : rltk::FontCharType;
        let name : String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = rltk::to_cp437('g'); name = "Goblin".to_string(); }
            _ => { glyph = rltk::to_cp437('o'); name = "Orc".to_string(); }
        }

        gs.ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .with(Name{ name: format!("{} #{}", &name, i) })
            .build();
    }

    gs.ecs.insert(map);

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Name{ name: "Player".to_string() })
        .with(Viewshed{ visible_tiles : Vec::new(), range : 8, dirty: true })  // TODO: 8 is a magic number
        .build();

    rltk::main_loop(context, gs)
}

