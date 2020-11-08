use crate::{CairoTextBox, Config, Alignment};
use std::fs::File;
use crate::config::*;
use std::io::{BufRead, BufReader};
use crate::utils;
use super::BarModule;

pub struct Battery {
    pub config: Config,
    pub dirs: Vec<String>,
}

impl BarModule for Battery {
    fn render(&self, cairo: &cairo::Context, mut align: f64) -> f64 {
        let mut i = 0;
        for d in &self.dirs {
            let f = File::open(d.clone() + "capacity")
                .expect("failed to open battery capacity file");
            let line = BufReader::new(f).lines().next().unwrap().unwrap();
            // sysfs sometimes gives > 100 percent
            let percentage = std::cmp::min(line.parse::<i32>().unwrap(), 100);
            let p = percentage as f64 / 100.0;

            // using only half the margin from the second battery on
            // this looks slightly better
            let margin: f64;
            if i > 0 {
                margin = BLOCK_MARGIN / 2.0;
            } else {
                margin = BLOCK_MARGIN;
            }

            // battery symbol
            let bat_sym_h = 0.6 * self.config.height;
            let bat_sym_w = 25.0;
            let bat_sym_margin = 3.0;
            let bat_sym_left = align - (bat_sym_w + 2.0*bat_sym_margin) - margin;

            // background
            utils::cairo_source_rgb_hex(cairo, COLOR_BG_BATTERY);
            cairo.rectangle(bat_sym_left,
                            0.0,
                            bat_sym_w + 2.0 * bat_sym_margin + margin,
                            self.config.height);
            cairo.fill();
            // battery background
            utils::cairo_source_rgb_hex(cairo, 0x0);
            cairo.rectangle(align - (bat_sym_w + bat_sym_margin) - margin,
                            0.5 * (self.config.height - bat_sym_h),
                            bat_sym_w,
                            bat_sym_h);
            cairo.fill();
            // battery "connector piece"
            cairo.rectangle(align - (bat_sym_margin) - margin,
                            0.5 * (self.config.height - bat_sym_h * 0.5),
                            3.0,
                            bat_sym_h * 0.5);
            cairo.fill();
            // faded inner color
            utils::cairo_source_rgb_rgfade(cairo, p);
            cairo.rectangle(align - (bat_sym_w + bat_sym_margin) + 3.0 - margin,
                            0.5 * (self.config.height - bat_sym_h) + 3.0,
                            (bat_sym_w - 6.0) * p,
                            bat_sym_h - 6.0);
            cairo.fill();

            let b = CairoTextBox {
                text: format!("{}%", percentage),
                height: self.config.height,
                color_text: COLOR_TEXT,
                color_box: COLOR_BG_BATTERY,
                alignment: Alignment::Right,
                align: bat_sym_left,
                margin: BLOCK_MARGIN,
            };
            align = b.draw(cairo);
            i += 1;
        }
        align 
    }
}
