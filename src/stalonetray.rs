use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::BarState;
use crate::config::*;

pub fn run(bar_state: Arc<(Mutex<BarState>,Condvar)>) {
    let cmd;
    {
        let b = bar_state.0.lock().unwrap();
        // TODO: how to calculate alignment position? (-1920)
        cmd = format!("stalonetray \
                      --icon-size {} \
                      --background \"#{:x}\" \
                      --grow-gravity E \
                      --geometry 1x1-{}+0 \
                      --kludges force_icons_size \
                      --log-level info 2>&1",
                      b.dyn_config.height,
                      COLOR_BG_STALONETRAY,
                      b.dyn_config.stalone_offset);
    }
    let re = Regex::new(r"geometry: \d+x\d+\+(\d+)*").unwrap();
    thread::spawn(move || {
        let out = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute command")
            .stdout
            .expect("failed to execute command");
        let reader = BufReader::new(out);
        reader.lines()
            .filter_map(|line| line.ok())
            .for_each(|line| {
                for c in re.captures_iter(&line.to_string()) {
                    // resize the bar to the new stalonetray width
                    let w = &c[1].parse::<f64>().ok().unwrap();
                    let mut b = bar_state.0.lock().unwrap();
                    b.dyn_config.width = *w;
                }
            });
    });
}
