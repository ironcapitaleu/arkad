#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during internal state operations.")
    }
}

impl std::error::Error for State {}
