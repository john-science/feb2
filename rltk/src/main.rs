use rltk::{GameState, Point, Rltk, RltkBuilder};
use specs::prelude::*;

mod components;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod rect;
mod spawner;
mod visibility_system;

pub use components::*;
pub use damage_system::*;
pub use gamelog::*;
pub use gui::*;
pub use inventory_system::*;
pub use map::*;
pub use map_indexing_system::*;
pub use melee_combat_system::MeleeCombatSystem;
pub use monster_ai_system::MonsterAI;
use player::*;
pub use rect::Rect;
pub use spawner::*;
use visibility_system::VisibilitySystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }

pub struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        // handle the FOV visibilityDiaganoal directions and
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }

        // draw the UI panel
        gui::draw_ui(&self.ecs, ctx);
    }
}


fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("February Second")
        .build()?;

    let mut gs = State{
        ecs: World::new()
    };
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();  // TODO: Is it possible in ECS to say there can be only one of these?
    gs.ecs.register::<Position>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
   
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog{ entries : vec!["You are dead. You wake up in Purgatory.".to_string()] });

    rltk::main_loop(context, gs)
}

