use crate::config::*;
use crate::utils::*;
use crate::BarState;
use crate::{CairoTextBox, DynamicConfig, Alignment};
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;
use chrono::Local;
use super::BarModule;

pub struct Clock {}

impl BarModule for Clock {
    fn render(&self, dyn_config: DynamicConfig, cairo: &cairo::Context, align: f64) -> f64 {
        let date = Local::now();
        let time_str = format!("{}", date.format(DATE_FORMAT));
        let b = CairoTextBox {
            text: time_str,
            height: dyn_config.height,
            color_text: COLOR_TEXT,
            color_box: COLOR_BG_CLOCK,
            alignment: Alignment::Right,
            align: align,
            margin: BLOCK_MARGIN,
        };
        b.draw(cairo)
    }

    fn event_generator(&self, bar_state: Arc<(Mutex<BarState>, Condvar)>) {
        thread::spawn(move || {
            loop {
                signal_bar_redraw(bar_state.clone());
                thread::sleep(Duration::from_millis(INTERVAL));
            }
        });
    }
}
