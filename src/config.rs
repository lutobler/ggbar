pub const FONT: &str                             = "Inconsolata Bold 11";
pub const DATE_FORMAT: &str                      = "%d.%m.%Y [%H:%M:%S]";
pub const INTERVAL: u64                          = 500; // milliseconds
pub const BLOCK_MARGIN: f64                      = 15.0;
pub const TAG_MARGIN: f64                        = 8.0;
pub const TAG_SPACE: f64                         = 0.0;

// monitor focus square size in % of height
pub const MONITOR_FOCUS_SIZE: f64                = 0.5;

pub const COLOR_BG: u32                          = 0x333333;
pub const COLOR_TEXT: u32                        = 0xdfdfdf;
pub const COLOR_BG_CLOCK: u32                    = 0x606060;

// tag status
pub const COLOR_EMPTY: u32                       = 0x606060;
pub const COLOR_NON_EMPTY: u32                   = 0x7e2d71;
pub const COLOR_THIS_MONITOR_UNFOCUSED: u32      = 0x2399d7;
pub const COLOR_THIS_MONITOR_FOCUSED: u32        = 0x2399d7;
pub const COLOR_DIFFERENT_MONITOR_UNFOCUSED: u32 = 0x308b55;
pub const COLOR_DIFFERENT_MONITOR_FOCUSED: u32   = 0x308b55;
pub const COLOR_URGENT_WINDOW: u32               = 0xff0000;

// monitor focus
pub const COLOR_MONITOR_FOCUSED: u32             = 0x2399d7;
pub const COLOR_MONITOR_UNFOCUSED: u32           = 0xdfdfdf;
