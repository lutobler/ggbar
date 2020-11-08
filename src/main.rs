use std::thread;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, Condvar};
use std::io::{BufRead, BufReader};

mod config;
use config::*;

mod modules;
use modules::BarModule;
use modules::basebar;
use modules::herbstluftwm;
use modules::clock;
use modules::battery;

mod utils;
use utils::*;

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

        // render modules
        for m in bar_state.modules_global.iter() {
            m.render(&bar_state.cairo, 0.0);
        }

        let mut l = 0.0;
        for m in bar_state.modules_left.iter() {
            l = m.render(&bar_state.cairo, l) + BLOCK_SPACE;
        }
        
        let mut r = bar_state.config.width;
        for m in bar_state.modules_right.iter() {
            r = m.render(&bar_state.cairo, r) - BLOCK_SPACE;
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
pub struct Config {
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
    modules_left: Vec<Box<dyn BarModule>>,
    modules_right: Vec<Box<dyn BarModule>>,
    modules_global: Vec<Box<dyn BarModule>>,
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
    // we can't draw on the window directly, we need the double buffering
    // from xcb_copy
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
        modules_left:   vec![
            Box::new(herbstluftwm::HerbstluftWM { config }),
        ],
        modules_right:  vec![
            Box::new(clock::Clock { config: config }),
            Box::new(battery::Battery {
                config: config,
                dirs: vec![
                    String::from("/sys/class/power_supply/BAT0/"),
                    String::from("/sys/class/power_supply/BAT1/"),
                ],
            }),
        ],
        modules_global: vec![ Box::new(basebar::BaseBar {}), ],
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
