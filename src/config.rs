use crate::modules::*;

pub const STALONETRAY_ENABLED: bool              = true;
pub const COLOR_BG_STALONETRAY: u32              = 0x606060;

pub const FONT: &str                             = "Inconsolata Bold 17";
pub const DATE_FORMAT: &str                      = "%a %d.%m.%Y [%H:%M:%S]";
pub const INTERVAL: u64                          = 500; // milliseconds
pub const BLOCK_MARGIN: f64                      = 10.0;
pub const TAG_MARGIN: f64                        = 8.0;
pub const TAG_SPACE: f64                         = 0.0;
pub const BLOCK_SPACE: f64                       = 0.0;

// hlwm monitor focus square size in % of height
pub const MONITOR_FOCUS_SIZE: f64                = 0.5;

pub const COLOR_BG: u32                          = 0x333333;
pub const COLOR_TEXT: u32                        = 0xdfdfdf;
pub const COLOR_BG_CLOCK: u32                    = 0x606060;
pub const COLOR_BG_BATTERY: u32                  = 0x808080;

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


// bar modules
pub fn modules_global() -> Vec<Box<dyn BarModule>> {
    vec![ Box::new(basebar::BaseBar{}) ]
}

pub fn modules_left() -> Vec<Box<dyn BarModule>> {
    vec![ Box::new(herbstluftwm::HerbstluftWM{}), ]
}

pub fn modules_right() -> Vec<Box<dyn BarModule>> {
    vec![
        Box::new(clock::Clock{}),
        // Box::new(battery::Battery {
        //     dirs: vec![
        //         String::from("/sys/class/power_supply/BAT0/"),
        //         String::from("/sys/class/power_supply/BAT1/"),
        //     ],
        // }),
    ]
}
