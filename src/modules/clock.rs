use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;
use chrono::Local;
use crate::config::*;
use crate::utils::*;
use crate::{CairoTextBox, DynamicConfig, Alignment};
use super::BarModule;

pub struct Clock {
    pub config: DynamicConfig
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

    fn event_generator(&self, sync: Arc<(Mutex<bool>, Condvar)>) {
        thread::spawn(move || {
            loop {
                signal_mutex(&sync.0, &sync.1);
                thread::sleep(Duration::from_millis(INTERVAL));
            }
        });
    }
}
