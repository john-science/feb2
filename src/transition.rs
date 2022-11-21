/*
  TODO
 */


pub struct Transition {
    pub level: u32,
    pub value: u32,
}


// Returns a value that depends on level. the table specifies what
// value occurs after each level, default is 0.
pub fn from_map_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}
