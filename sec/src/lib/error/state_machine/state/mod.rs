#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Invalid Cik format.
    InvalidCikFormat,

    /// Indicates that input data of a `State` is invalid and cannot be used to compute the output data.
    InvalidInputData,

    /// Indicates that context data of a `State` is invalid and cannot be used to compute the output data.
    InvalidContextData,

    /// Indicates that the output computation of a `State` has failed.
    FailedOutputComputation,

    /// Indicates a failure during the update of the internal `StateData` of the `State`.
    StateDataUpdateFailed,

    /// Indicates a failure during the update of the `ContextData` of the `State`.
    ContextDataUpdateFailed,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during internal state operations.")
    }
}

impl std::error::Error for State {}
