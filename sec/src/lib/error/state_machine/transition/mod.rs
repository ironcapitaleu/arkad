#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    /// Failed to convert output of the source state into the input of the destination state.
    FailedOutputConversion,

    /// Failed to convert context of the source state into the context of the destination state.
    FailedContextConversion,
}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during transition operations.")
    }
}

impl std::error::Error for Transition {}
