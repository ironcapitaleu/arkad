use std::fmt::Debug;

/// A trait representing the behavior of an inner channel.
pub trait InnerChannel: Send + Sync + 'static + Debug {
    fn serve(&self);
}
