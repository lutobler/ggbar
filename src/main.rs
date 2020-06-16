extern crate xcb;
extern crate cairo_sys;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate chrono;

use chrono::Local;
use std::thread;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, Condvar};
use std::io::{BufRead, BufReader};

mod config;
use config::*;

trait Renderable {
    fn render(&self, cairo: &cairo::Context);
}

/*
 * Bar modules
 */
struct BaseBar {}                       // bar background
struct HerbstluftWM { config: Config }  // hlwm tag indicator
struct Clock { config: Config }         // clock

impl Renderable for BaseBar {
    fn render(&self, cairo: &cairo::Context) {
        cairo_source_rgb_hex(cairo, COLOR_BG);
        cairo.paint();
    }
}

impl Renderable for HerbstluftWM {
    fn render(&self, cairo: &cairo::Context) {
        let tags = Tag::read_hlwm_tags(self.config.monitor);

        // monitor focus status square
        let mut focus_color = COLOR_MONITOR_UNFOCUSED;
        for t in tags.iter() {
            if t.state == TagState::ThisMonitorFocused {
                focus_color = COLOR_MONITOR_FOCUSED;
            }
        }
        cairo_source_rgb_hex(cairo, focus_color);
        let focus_state_w = self.config.height;
        let h = self.config.height;
        let focus_margin = 0.5 * (h - (h * MONITOR_FOCUS_SIZE));
        cairo.rectangle(focus_margin,
                        focus_margin,
                        focus_state_w - 2.0*focus_margin,
                        focus_state_w - 2.0*focus_margin);
        cairo.fill();

        // herstluftwm tags
        let mut left_border: f64 = focus_state_w;
        for t in tags {
            let text = format!("{}", t.name);
            let b = CairoTextBox {
                text: text,
                height: self.config.height,
                color_text: COLOR_TEXT,
                color_box: t.state.color(),
                alignment: Alignment::Left,
                align: left_border,
                margin: TAG_MARGIN,
            };
            let new_left = b.draw(cairo);
            left_border = new_left + TAG_SPACE;
        }
    }
}

impl Renderable for Clock {
    fn render(&self, cairo: &cairo::Context) {
        let date = Local::now();
        let time_str = format!("{}", date.format(DATE_FORMAT));
        let b = CairoTextBox {
            text: time_str,
            height: self.config.height,
            color_text: COLOR_TEXT,
            color_box: COLOR_BG_CLOCK,
            alignment: Alignment::Right,
            align: self.config.width,
            margin: BLOCK_MARGIN,
        };
        b.draw(cairo);
    }
}

/*
 * HLWM tag parsing
 */

#[derive(PartialEq)]
enum TagState {
    Empty,
    NonEmpty,
    ThisMonitorUnfocused,
    ThisMonitorFocused,
    DifferentMonitorUnfocused,
    DifferentMonitorFocused,
    UrgentWindow,
}

struct Tag {
    state: TagState,
    name: String,
}

impl TagState {
    fn from_symbol(sym: char) -> Option<TagState> {
        match sym {
            '.' => Some(TagState::Empty),
            ':' => Some(TagState::NonEmpty),
            '+' => Some(TagState::ThisMonitorUnfocused),
            '#' => Some(TagState::ThisMonitorFocused),
            '-' => Some(TagState::DifferentMonitorUnfocused),
            '%' => Some(TagState::DifferentMonitorFocused),
            '!' => Some(TagState::UrgentWindow),
            _ => None
        }
    }

    fn color(&self) -> u32 {
        match self {
            TagState::Empty => COLOR_EMPTY,
            TagState::NonEmpty => COLOR_NON_EMPTY,
            TagState::ThisMonitorUnfocused => COLOR_THIS_MONITOR_UNFOCUSED,
            TagState::ThisMonitorFocused => COLOR_THIS_MONITOR_FOCUSED,
            TagState::DifferentMonitorUnfocused => COLOR_DIFFERENT_MONITOR_UNFOCUSED,
            TagState::DifferentMonitorFocused => COLOR_DIFFERENT_MONITOR_FOCUSED,
            TagState::UrgentWindow => COLOR_URGENT_WINDOW,
        }
    }
}

impl Tag {
    fn from_str(s: &str) -> Option<Tag> {
        if s.len() == 0 {
            return None;
        }
        let tag_state: TagState;
        let tag_state_opt = TagState::from_symbol(s[0..].chars().next().unwrap());
        match tag_state_opt {
            Some(ts) => tag_state = ts,
            None => return None
        }

        Some(Tag {
            state: tag_state,
            name: String::from(&s[1..]),
        })
    }

    fn read_hlwm_tags(monitor: i32) -> Vec<Tag> {
        let hc_output = Command::new("/usr/bin/herbstclient")
            .arg("tag_status")
            .arg(monitor.to_string())
            .output()
            .expect("failed to execute command");
        let tags_hc_out = String::from_utf8_lossy(&hc_output.stdout);
        let tags_str: Vec<&str> = tags_hc_out.split('\t').collect();
        let mut tags: Vec<Tag> = Vec::new();
        for t in tags_str {
            let t_opt = Tag::from_str(t);
            match t_opt {
                Some(tt) => tags.push(tt),
                None => continue,
            }
        }
        return tags
    }
}

/*
 * Cairo drawing functions
 */
fn cairo_source_rgb_hex(cairo: &cairo::Context, color: u32) {
    cairo.set_source_rgb(
        ((color >> 16) & 0xff) as f64 / 255.0,
        ((color >> 8) & 0xff) as f64 / 255.0,
        ((color >> 0) & 0xff) as f64 / 255.0);
}

fn setup_pango_layout(cairo: &cairo::Context) -> pango::Layout {
    let pango_layout = pangocairo::create_layout(&cairo)
        .expect("failed create pango layout");
    let font_description = pango::FontDescription::from_string(FONT);
    pango_layout.set_font_description(Some(&font_description));
    return pango_layout
}

enum Alignment {
    Left,
    Right
}

struct CairoTextBox {
    text: String,
    height: f64,
    color_text: u32,
    color_box: u32,
    alignment: Alignment,
    align: f64,
    margin: f64,
}

impl CairoTextBox {
    fn draw(&self, cairo: &cairo::Context) -> f64 {
        let pl = setup_pango_layout(cairo);
        pl.set_text(self.text.as_str());
        let (w, h) = pl.get_size();
        let w_text = (w / pango::SCALE) as f64;
        let h_text = (h / pango::SCALE) as f64;
        let w_margins = w_text + 2.0 * self.margin;

        let left;
        match self.alignment {
            Alignment::Left => left = self.align,
            Alignment::Right => left = self.align - w_margins,
        }

        // background
        cairo_source_rgb_hex(cairo, self.color_box);
        cairo.rectangle(left, 0.0, w_margins, self.height);
        cairo.fill();

        // text
        cairo_source_rgb_hex(cairo, self.color_text);
        cairo.move_to(left + self.margin, 0.5 * (self.height - h_text));
        pangocairo::show_layout(&cairo, &pl);

        match self.alignment {
            Alignment::Left => left + w_margins,
            Alignment::Right => left,
        }
    }
}

fn draw_thread(bar_state: &BarState, lock: &Mutex<bool>, cvar: &Condvar) {
    loop {
        let mut signaled = lock.lock().unwrap();
        while !*signaled {
            signaled = cvar.wait(signaled).unwrap();
        }

        // render all modules
        let modules = bar_state.modules_global.iter()
            .chain(bar_state.modules_left.iter())
            .chain(bar_state.modules_right.iter());
        for m in modules {
            m.render(&bar_state.cairo);
        }

        xcb::xproto::copy_area(&bar_state.connection,
                               bar_state.pixmap,
                               bar_state.window,
                               bar_state.gcontext,
                               0, 0, 0, 0,
                               bar_state.config.width as u16,
                               bar_state.config.height as u16);
        bar_state.connection.flush();

        *signaled = false;
        drop(signaled);
    }
}

fn signal_mutex(lock: &Mutex<bool>, cvar: &Condvar) {
    let mut signaled = lock.lock().unwrap();
    *signaled = true;
    cvar.notify_one();
    drop(signaled);
}

fn periodic_event_generator(lock: &Mutex<bool>, cvar: &Condvar) {
    loop {
        signal_mutex(lock, cvar);
        thread::sleep(Duration::from_millis(INTERVAL));
    }
}

fn herbstluftwm_event_generator(lock: &Mutex<bool>, cvar: &Condvar) {
    loop {
        let hc_output = Command::new("/usr/bin/herbstclient")
            .arg("-i")
            .arg("tag_changed|tag_renamed")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute command")
            .stdout
            .expect("failed to execute command");
        let reader = BufReader::new(hc_output);
        reader.lines()
            .filter_map(|line| line.ok())
            .for_each(|_| {
                signal_mutex(lock, cvar)
            });
    }
}

// non-static configuration (given as arg)
#[derive(Copy, Clone, Default)]
struct Config {
    x_offset: f64,
    y_offset: f64,
    width: f64,
    height: f64,
    monitor: i32,
}

struct BarState {
    cairo: cairo::Context,
    connection: xcb::Connection,
    window: xcb::xproto::Window,
    pixmap: xcb::xproto::Pixmap,
    gcontext: xcb::xproto::Gcontext,
    config: Config,
    modules_left: Vec<Box<dyn Renderable>>,
    modules_right: Vec<Box<dyn Renderable>>,
    modules_global: Vec<Box<dyn Renderable>>,
}
unsafe impl Send for BarState {}

fn get_root_visual_type(screen: &xcb::Screen) -> xcb::Visualtype {
    for depth in screen.allowed_depths() {
        for visual in depth.visuals() {
            if screen.root_visual() == visual.visual_id() {
                return visual;
            }
        }
    }
    panic!("no visual type found");
}

fn main() {
    // parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config: Config = Default::default();
    match args.len() {
        6 => {
            config.x_offset = args[1].parse::<f64>().unwrap();
            config.y_offset = args[2].parse::<f64>().unwrap();
            config.width = args[3].parse::<f64>().unwrap();
            config.height = args[4].parse::<f64>().unwrap();
            config.monitor = args[5].parse::<i32>().unwrap();
        },
        _ => panic!("wrong number of arguments"),
    }

    // set up xcb
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let win = conn.generate_id();

    let ev_mask = xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_BUTTON_PRESS;
    let value_list = &[
        (xcb::CW_EVENT_MASK, ev_mask),
        (xcb::CW_OVERRIDE_REDIRECT, 1)
    ];
    xcb::create_window(&conn,
                       xcb::COPY_FROM_PARENT as u8,
                       win,
                       screen.root(),
                       config.x_offset as i16,
                       config.y_offset as i16,
                       config.width as u16,
                       config.height as u16,
                       0,
                       xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                       screen.root_visual(),
                       value_list);
    xcb::map_window(&conn, win);
    conn.flush();

    // set up pixmap
    let pixmap: xcb::xproto::Pixmap = conn.generate_id();
    xcb::xproto::create_pixmap(&conn,
                               screen.root_depth(),
                               pixmap,
                               screen.root(),
                               config.width as u16,
                               config.height as u16);

    // set up graphics context
    let gcontext = conn.generate_id();
    let gc_value_list = &[
        (xcb::GC_FOREGROUND, screen.black_pixel()),
        (xcb::GC_GRAPHICS_EXPOSURES, 0)
    ];
    xcb::create_gc(&conn, gcontext, win, gc_value_list);

    // set up cairo
    let root_visual_type = &mut get_root_visual_type(&screen).base
        as *mut _ as *mut cairo_sys::xcb_visualtype_t;
    let raw_cairo_conn = conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t;

    let cairo_conn;
    let visual_type;
    unsafe {
        cairo_conn = cairo::XCBConnection::from_raw_none(raw_cairo_conn);
        visual_type = cairo::XCBVisualType::from_raw_none(root_visual_type);
    }

    let surface = cairo::XCBSurface::create(&cairo_conn,
                                            &cairo::XCBDrawable(pixmap),
                                            &visual_type,
                                            config.width as i32,
                                            config.height as i32)
        .expect("failed to create XCBSurface");
    let cr = cairo::Context::new(&surface);

    // modules and bar state
    let bar_state = BarState {
        cairo:          cr,
        connection:     conn,
        window:         win,
        pixmap:         pixmap,
        gcontext:       gcontext,
        config:         config,
        modules_left:   vec![ Box::new(HerbstluftWM { config }), ],
        modules_right:  vec![ Box::new(Clock { config }), ],
        modules_global: vec![ Box::new(BaseBar {}), ],
    };

    // event generators
    let event_generators: Vec<fn(&Mutex<bool>,&Condvar)> = vec![
        periodic_event_generator,
        herbstluftwm_event_generator
    ];

    let p0 = Arc::new((Mutex::new(false), Condvar::new()));

    // start event generators
    for ev_g in event_generators {
        let pc = p0.clone();
        thread::spawn(move || {
            ev_g(&pc.0, &pc.1);
        });
    }

    // start drawing thread
    let draw_thread_handler = thread::spawn(move || {
        draw_thread(&bar_state, &p0.0, &p0.1);
    });
    draw_thread_handler.join().unwrap();
}
