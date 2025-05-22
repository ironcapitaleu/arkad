#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Invalid Cik.
    InvalidCikFormat,

    /// This is the failure updating the internal `StateData` of the `State`.
    StateDataUpdateFailed,

    ///  This is the failure updating the `ContextData` of the `State`.
    ContextDataUpdateFailed,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during internal state operations.")
    }
}

impl std::error::Error for State {}
