#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecError {
    InvalidCikFormat(String),
}