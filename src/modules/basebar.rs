use crate::config::*;
use crate::utils;
use super::BarModule;

pub struct BaseBar {}

impl BarModule for BaseBar {
    fn render(&self, cairo: &cairo::Context, align: f64) -> f64 {
        utils::cairo_source_rgb_hex(cairo, COLOR_BG);
        cairo.paint();
        align
    }
}
