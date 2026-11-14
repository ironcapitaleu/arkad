/// Human-readable name of the "Store Full Index" state, used in error messages and logging.
pub const STATE_NAME: &str = "Store Full Index";

/// The start of the key of a stored full index, before `YYYY/QTRn/`.
pub const FULL_INDEX_KEY_PREFIX: &str = "sec/full-index/";

/// The file name of a stored full index, after `YYYY/QTRn/`. It is the SEC file name.
pub const FULL_INDEX_KEY_FILE_NAME: &str = "form.idx";
