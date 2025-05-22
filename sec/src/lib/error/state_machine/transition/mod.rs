#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during transition operations.")
    }
}

impl std::error::Error for Transition {}
