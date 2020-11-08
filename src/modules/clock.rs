use crate::{CairoTextBox, Config, Alignment};
use chrono::Local;
use crate::config::*;
use super::BarModule;

pub struct Clock {
    pub config: Config
}

impl BarModule for Clock {
    fn render(&self, cairo: &cairo::Context, align: f64) -> f64 {
        let date = Local::now();
        let time_str = format!("{}", date.format(DATE_FORMAT));
        let b = CairoTextBox {
            text: time_str,
            height: self.config.height,
            color_text: COLOR_TEXT,
            color_box: COLOR_BG_CLOCK,
            alignment: Alignment::Right,
            align: align,
            margin: BLOCK_MARGIN,
        };
        b.draw(cairo)
    }
}
