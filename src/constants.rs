/*
  A central place for general constants.
 */
use tcod::colors::Color;

// Top-level strings
pub const GAME_TITLE: &str = "February Second";
pub const AUTHOR_LINE: &str = "by John Science";
pub const FONT_IMG: &str = "dejavu16x16.png";

// player will always be the first object
pub const PLAYER: usize = 0;

// actual size of the window
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;

// size of the map
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;

// field-of-view
pub const TORCH_RADIUS: i32 = 10;

// colors for map objects
pub const COLOR_DARK_WALL: Color = Color { r: 6, g: 3, b: 1 };
pub const COLOR_DARK_GROUND: Color = Color { r: 81, g: 44, b: 15 };
pub const COLOR_LIGHT_WALL: Color = Color { r: 30, g: 16, b: 5 };
pub const COLOR_LIGHT_GROUND: Color = Color { r: 124, g: 65, b: 21 };

// sizes and coordinates relevant for the GUI
pub const BAR_WIDTH: i32 = 20;
pub const PANEL_HEIGHT: i32 = 7;
pub const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
pub const MSG_X: i32 = BAR_WIDTH + 2;
pub const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
pub const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
pub const CHARACTER_SCREEN_WIDTH: i32 = 30;
pub const INVENTORY_WIDTH: i32 = 50;

// TODO: Should depend on skills
// spells and magic
pub const HEAL_AMOUNT: i32 = 40;
pub const LIGHTNING_DAMAGE: i32 = 40;
pub const LIGHTNING_RANGE: i32 = 5;
pub const CONFUSE_RANGE: i32 = 8;
pub const CONFUSE_NUM_TURNS: i32 = 10;
pub const FIREBALL_RADIUS: i32 = 3;
pub const FIREBALL_DAMAGE: i32 = 25;

// parameters for map generator
pub const ROOM_MAX_SIZE: i32 = 12;
pub const ROOM_MIN_SIZE: i32 = 6;
pub const MAX_ROOMS: i32 = 32;

// parameters for game size
pub const MAX_LVL: u32 = 21;  // TODO: Game balance
pub const KARMA_TO_ASCEND: i32 = 1000;  // TODO: Game balance
