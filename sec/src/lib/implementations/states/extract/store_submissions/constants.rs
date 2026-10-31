/// Human-readable name of the "Store Submissions" state, used in error messages and logging.
pub const STATE_NAME: &str = "Store Submissions";

/// The start of the key of a stored `submissions` document, before the 10-digit CIK.
pub const SUBMISSIONS_KEY_PREFIX: &str = "sec/submissions/CIK";

/// The end of the key of a stored `submissions` document, after the 10-digit CIK.
pub const SUBMISSIONS_KEY_SUFFIX: &str = ".json";
