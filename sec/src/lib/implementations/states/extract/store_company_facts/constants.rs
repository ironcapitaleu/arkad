/// Human-readable name of the "Store Company Facts" state, used in error messages and logging.
pub const STATE_NAME: &str = "Store Company Facts";

/// The start of the key of a stored Company Facts document, before the 10-digit CIK.
pub const COMPANY_FACTS_KEY_PREFIX: &str = "sec/companyfacts/CIK";

/// The end of the key of a stored Company Facts document, after the 10-digit CIK.
pub const COMPANY_FACTS_KEY_SUFFIX: &str = ".json";
