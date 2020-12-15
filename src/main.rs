use std::thread;
use std::sync::{Arc, Mutex, Condvar};

mod config;
use config::*;

mod modules;
use modules::BarModule;

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

// non-static configuration (given as arg)
#[derive(Copy, Clone, Default)]
pub struct DynamicConfig {
    x_offset: f64,
    y_offset: f64,
    width: f64,
    height: f64,
    monitor: i32,
}

struct BarState {
    cairo: cairo::Context,
    connection: Arc<xcb::Connection>,
    window: xcb::xproto::Window,
    pixmap: xcb::xproto::Pixmap,
    gcontext: xcb::xproto::Gcontext,
    config: DynamicConfig,
    modules_left: Vec<Box<dyn BarModule>>,
    modules_right: Vec<Box<dyn BarModule>>,
    modules_global: Vec<Box<dyn BarModule>>,
}
unsafe impl Send for BarState {}

fn main() {
    // parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut dyn_config: DynamicConfig = Default::default();
    match args.len() {
        6 => {
            dyn_config.x_offset = args[1].parse::<f64>().unwrap();
            dyn_config.y_offset = args[2].parse::<f64>().unwrap();
            dyn_config.width = args[3].parse::<f64>().unwrap();
            dyn_config.height = args[4].parse::<f64>().unwrap();
            dyn_config.monitor = args[5].parse::<i32>().unwrap();
        },
        _ => panic!("wrong number of arguments"),
    }

    // set up xcb
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let win = conn.generate_id();

    let ev_mask = xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS;
    let value_list = &[
        (xcb::CW_EVENT_MASK, ev_mask),
        (xcb::CW_OVERRIDE_REDIRECT, 1)
    ];
    xcb::create_window(&conn,
                       xcb::COPY_FROM_PARENT as u8,
                       win,
                       screen.root(),
                       dyn_config.x_offset as i16,
                       dyn_config.y_offset as i16,
                       dyn_config.width as u16,
                       dyn_config.height as u16,
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
                               dyn_config.width as u16,
                               dyn_config.height as u16);

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
                                            dyn_config.width as i32,
                                            dyn_config.height as i32)
        .expect("failed to create XCBSurface");
    let cr = cairo::Context::new(&surface);

    let conn_arc = Arc::new(conn);

    // modules and bar state
    let bar_state = BarState {
        cairo:          cr,
        connection:     conn_arc.clone(),
        window:         win,
        pixmap:         pixmap,
        gcontext:       gcontext,
        config:         dyn_config,
        modules_left:   modules_left(dyn_config),
        modules_right:  modules_right(dyn_config),
        modules_global: modules_global(dyn_config),
    };

    let sync = Arc::new((Mutex::new(false), Condvar::new()));

    // start event generators
    let all_modules = bar_state.modules_global.iter().chain(
        bar_state.modules_left.iter().chain(
            bar_state.modules_right.iter()));
    for m in all_modules {
        let s = sync.clone();
        m.event_generator(s);
    }

    let s = sync.clone();
    let draw_thread_handler = thread::spawn(move || {
        draw_thread(&bar_state, &s.0, &s.1);
    });

    // deal with X events in the main thread
    loop {
        let event = conn_arc.wait_for_event();
        match event {
            None => break,
            Some(event) => {
                let r = event.response_type() & !0x80;
                match r {
                    xcb::EXPOSE => {
                        signal_mutex(&sync.0, &sync.1);
                    }
                    xcb::KEY_PRESS => {
                    }
                    _ => {}
                }
            }
        }
    }

    draw_thread_handler.join().unwrap();
}
