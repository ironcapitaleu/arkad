use std::fmt::Debug;

/// A trait representing the behavior of an inner channel.
pub trait InnerChannel: Send + Sync + 'static + Debug {
    fn serve(&self);
}

// TODO: remove this later and put it in the right place
use lapin::Channel as LapinChannel;
// Implementation of InnerChannel for LapinChannel
impl InnerChannel for LapinChannel {
    fn serve(&self) {
        println!("LapinChannel is serving.");
    }
}
