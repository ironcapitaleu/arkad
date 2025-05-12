use state_maschine::prelude::*;

pub trait SecContextData: ContextData {
    fn can_retry(&self) -> bool {
        self.get_retries() > 0
    }

    fn get_retries(&self) -> u32;
}
