use std::sync::{Arc, Mutex, Condvar};

pub trait BarModule {
    fn render(&self, cairo: &cairo::Context, align: f64) -> f64;
    fn event_generator(&self, sync: Arc<(Mutex<bool>, Condvar)>);
}

pub mod basebar;
pub mod herbstluftwm;
pub mod clock;
pub mod battery;
