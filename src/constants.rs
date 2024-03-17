pub const IS_TESTING: bool = false;

#[cfg(debug_assertions)]
pub const AUTO_OPTIMIZE: bool = false;
#[cfg(not(debug_assertions))]
pub const AUTO_OPTIMIZE: bool = true;

pub const STATE_ARRAY_SIZE: usize = 50;
pub const DEFAULT_STATE_SYMBOL: char = 'q';
