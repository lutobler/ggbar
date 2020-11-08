use crate::config::*;

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

pub fn setup_pango_layout(cairo: &cairo::Context) -> pango::Layout {
    let pango_layout = pangocairo::create_layout(&cairo)
        .expect("failed create pango layout");
    let font_description = pango::FontDescription::from_string(FONT);
    pango_layout.set_font_description(Some(&font_description));
    return pango_layout
}
