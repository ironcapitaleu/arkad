use state_maschine::prelude::*;

pub trait SecContextData: ContextData {
    fn can_retry(&self) -> bool {
        self.get_max_retries() > 0
    }

    fn get_max_retries(&self) -> u32;
}
