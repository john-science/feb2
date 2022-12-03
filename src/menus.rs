/*
  General Menu Tools
 */
// Import Third-Party
use serde::{Deserialize, Serialize};
use tcod::colors::*;
use tcod::console::*;
use tcod::input::{Key, Mouse};
use tcod::map::{Map as FovMap};

// Import Locally
use crate::constants::INVENTORY_KEYS;
use crate::constants::INVENTORY_MAX;
use crate::constants::INVENTORY_WIDTH;
use crate::constants::SCREEN_HEIGHT;
use crate::constants::SCREEN_WIDTH;
use crate::objects::Object;


// TODO: Move to its own file?
pub struct Tcod {
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub fov: FovMap,
    pub key: Key,
    pub mouse: Mouse,
}


pub fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    // show a menu with each item of the inventory as an option
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory
            .iter()
            .map(|thing| {
                // show additional information, in case it's equipped
                match thing.equipment {
                    Some(equipment) if equipment.equipped => {
                        format!("{} (on {})", thing.name, equipment.slot)
                    }
                    _ => match thing.item {
                        Some(_item) if thing.charges > 1 => {
                            format!("{} ({})", thing.name, thing.charges)
                        }
                        _ => thing.name.clone(),
                    }
                }
            })
            .collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // if an item was chosen, return it
    if inventory.len() > 0 {
        return inventory_index;
    } else {
        return None;
    }
}


pub fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    assert!(
        options.len() <= INVENTORY_MAX,
        "Cannot have a menu with more than {} options.", INVENTORY_MAX
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        0
    } else {
        root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };
    let height = options.len() as i32 + header_height;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // print the header, with auto-wrap
    window.set_default_foreground(WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        //let menu_letter: char = (b'a' + index as u8) as char;
        let menu_letter: char = INVENTORY_KEYS.chars().nth(index).unwrap();
        let text: String = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit the contents of "window" to the root console
    let x: i32 = SCREEN_WIDTH / 2 - width / 2;
    let y: i32 = SCREEN_HEIGHT / 2 - height / 2;
    blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.9);

    // present the root console to the player and wait for a key-press
    root.flush();
    let key = root.wait_for_keypress(true);

    // convert the ASCII code to an index; if it corresponds to an option, return it
    return INVENTORY_KEYS.find(key.printable);
}


pub fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}


pub fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
    // render a bar (HP, experience, etc). First calculate the width of the bar
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    // render the background first
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    // now render the bar on top
    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    // finally, some centered text with the values
    panel.set_default_foreground(WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
}


#[derive(Serialize, Deserialize)]
pub struct Messages {
    messages: Vec<(String, Color)>,
}


impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    // add the new message as a tuple, with the text and the color
    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color));
    }

    // Create a `DoubleEndedIterator` over the messages
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
    }
}
