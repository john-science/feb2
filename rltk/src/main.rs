use rltk::{Rltk, RltkBuilder, GameState};

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "You awaken in Purgatory.");
        ctx.print(1, 2, "You mislived, you know why you're here.");
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("February Second")
        .build()?;
    let gs = State{ };
    rltk::main_loop(context, gs)
}

