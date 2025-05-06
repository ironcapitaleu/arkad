use state_maschine::prelude::*;

pub trait SecContextData: ContextData {
    fn can_retry(&self) -> bool;
    fn get_max_retries(&self) -> u32;
    fn get_current_retry(&self) -> u32;
    fn has_retries_left(&self) -> bool;
}
