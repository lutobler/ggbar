use crate::BarState;
use std::sync::{Arc, Mutex, Condvar};

pub fn cairo_source_rgb_hex(cairo: &cairo::Context, color: u32) {
    cairo.set_source_rgb(
        ((color >> 16) & 0xff) as f64 / 255.0,
        ((color >> 8) & 0xff) as f64 / 255.0,
        ((color >> 0) & 0xff) as f64 / 255.0);
}

// set the color to a fade from red to green for p in [0, 1]
pub fn cairo_source_rgb_rgfade(cairo: &cairo::Context, p: f64) {
    assert!(p >= 0.0 && p <= 1.0);
    if p <= 0.5 { // red to yellow
        cairo.set_source_rgb(1.0, 2.0*p, 0.0);
    } else { // yellow to green
        cairo.set_source_rgb(1.0 - 2.0*(p - 0.5), 1.0, 0.0);
    }
}

pub fn setup_pango_layout(cairo: &cairo::Context, font: String) -> pango::Layout {
    let pango_layout = pangocairo::create_layout(&cairo)
        .expect("failed create pango layout");
    let font_description = pango::FontDescription::from_string(&font[..]);
    pango_layout.set_font_description(Some(&font_description));
    return pango_layout
}

pub fn get_root_visual_type(screen: &xcb::Screen) -> xcb::Visualtype {
    for depth in screen.allowed_depths() {
        for visual in depth.visuals() {
            if screen.root_visual() == visual.visual_id() {
                return visual;
            }
        }
    }
    panic!("no visual type found");
}

pub fn signal_bar_redraw(bar_state: Arc<(Mutex<BarState>, Condvar)>) {
    let mut b = bar_state.0.lock().unwrap();
    let c = &bar_state.1;
    b.redraw_signaled = true;
    c.notify_one();
}
