// TCOD imports
use tcod::colors::*;
use tcod::console::*;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
}

fn main() {
    // initialize the TCOD "Root" object
    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("February Second")
        .init();

    // use the TCOD "Root" object to create a mutable TCOD struct
    let mut tcod = Tcod { root };

    // the game loop!
    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(WHITE);
        tcod.root.clear();
        tcod.root.put_char(1, 1, '@', BackgroundFlag::None);
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
    }
}

