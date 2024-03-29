use crate::{CairoTextBox, DynamicConfig, Alignment};
use crate::BarState;
use crate::config::*;
use crate::utils::*;
use crate::utils;
use std::sync::{Arc, Mutex, Condvar};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use super::BarModule;

pub struct HerbstluftWM {}

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

impl BarModule for HerbstluftWM {
    fn render(&self, dyn_config: DynamicConfig, cairo: &cairo::Context, align: f64) -> f64 {
        let tags = Tag::read_hlwm_tags(dyn_config.monitor);

        // monitor focus status square
        let mut focus_color = COLOR_MONITOR_UNFOCUSED;
        for t in tags.iter() {
            if t.state == TagState::ThisMonitorFocused {
                focus_color = COLOR_MONITOR_FOCUSED;
            }
        }
        utils::cairo_source_rgb_hex(cairo, focus_color);
        let focus_state_w = dyn_config.height;
        let h = dyn_config.height;
        let focus_margin = 0.5 * (h - (h * MONITOR_FOCUS_SIZE));
        cairo.rectangle(focus_margin + align,
                        focus_margin,
                        focus_state_w - 2.0*focus_margin,
                        focus_state_w - 2.0*focus_margin);
        cairo.fill();

        // herstluftwm tags
        let mut left_border: f64 = focus_state_w + align;
        for t in tags {
            let b = CairoTextBox {
                text: t.name,
                height: dyn_config.height,
                color_text: COLOR_TEXT,
                color_box: t.state.color(),
                alignment: Alignment::Left,
                align: left_border,
                margin: TAG_MARGIN,
            };
            let new_left = b.draw(cairo, dyn_config.clone().font);
            left_border = new_left + TAG_SPACE;
        }
        left_border
    }

    fn event_generator(&self, bar_state: Arc<(Mutex<BarState>, Condvar)>) {
        thread::spawn(move || {
            loop {
                let hc_output = Command::new("herbstclient")
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
                        signal_bar_redraw(bar_state.clone())
                    });
            }
        });
    }
}
