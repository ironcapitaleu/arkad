#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// The error enum for all queue errors.
///
/// This enum encapsulates all possible errors that can occur
/// during queue operations, including connection issues, configuration errors,
/// and operation failures.
pub enum ErrorKind {
    /// Invalid connector configuration
    InvalidConfiguration(String),
}
