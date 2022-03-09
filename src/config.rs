use crate::modules::*;

// pub const FONT: &str                = "Inconsolata Bold 18";
pub const DATE_FORMAT: &str         = "%a %d.%m.%Y [%H:%M:%S]";
pub const INTERVAL: u64             = 500; // milliseconds
pub const BLOCK_MARGIN: f64         = 10.0;
pub const TAG_MARGIN: f64           = 10.0;
pub const TAG_SPACE: f64            = 2.0;
pub const BLOCK_SPACE: f64          = 0.0;
pub const STALONETRAY_ENABLED: bool = true;

// hlwm monitor focus square size in % of height
pub const MONITOR_FOCUS_SIZE: f64   = 0.5;

pub const C_RED: u32    = 0xf2777a;
pub const C_GRAY1: u32  = 0x393939;
pub const C_GRAY3: u32  = 0x747369;
pub const C_GRAY4: u32  = 0xa09f93;
pub const C_PURPLE: u32 = 0xcc99cc;
pub const C_BLUE: u32   = 0x6699cc;
pub const C_GREEN: u32  = 0x99cc99;
pub const C_WHITE6: u32 = 0xe8e6df;

pub const COLOR_BG: u32                          = C_GRAY1;
pub const COLOR_TEXT: u32                        = C_WHITE6;
pub const COLOR_BG_CLOCK: u32                    = C_GRAY3;
pub const COLOR_BG_BATTERY: u32                  = C_GRAY4;
pub const COLOR_BG_STALONETRAY: u32              = C_GRAY3;

// hlwm tags
pub const COLOR_EMPTY: u32                       = C_GRAY3;
pub const COLOR_NON_EMPTY: u32                   = C_PURPLE;
pub const COLOR_THIS_MONITOR_UNFOCUSED: u32      = C_BLUE;
pub const COLOR_THIS_MONITOR_FOCUSED: u32        = C_BLUE;
pub const COLOR_DIFFERENT_MONITOR_UNFOCUSED: u32 = C_GREEN;
pub const COLOR_DIFFERENT_MONITOR_FOCUSED: u32   = C_GREEN;
pub const COLOR_URGENT_WINDOW: u32               = C_RED;

// monitor focus
pub const COLOR_MONITOR_FOCUSED: u32             = C_BLUE;
pub const COLOR_MONITOR_UNFOCUSED: u32           = C_WHITE6;


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
