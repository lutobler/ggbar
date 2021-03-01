use crate::DynamicConfig;
use crate::BarState;
use std::sync::{Arc, Mutex, Condvar};

pub trait BarModule {
    fn render(&self, dyn_config: DynamicConfig, cairo: &cairo::Context, align: f64) -> f64;
    fn event_generator(&self, sync: Arc<(Mutex<BarState>, Condvar)>);
}

pub mod basebar;
pub mod herbstluftwm;
pub mod clock;
pub mod battery;
