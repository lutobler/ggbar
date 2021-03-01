use crate::DynamicConfig;
use crate::BarState;
use crate::config::*;
use crate::utils;
use std::sync::{Arc, Mutex, Condvar};
use super::BarModule;

pub struct BaseBar {}

impl BarModule for BaseBar {
    fn render(&self, _dyn_config: DynamicConfig, cairo: &cairo::Context, align: f64) -> f64 {
        utils::cairo_source_rgb_hex(cairo, COLOR_BG);
        cairo.paint();
        align
    }

    fn event_generator(&self, _bar_state: Arc<(Mutex<BarState>, Condvar)>) {
        // no op
    }
}
