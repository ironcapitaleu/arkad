#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum State {
    /// Invalid Cik format.
    InvalidCikFormat(String),

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fmt::Debug, hash::Hash};
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_state() {
        implements_auto_traits::<State>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implementend_send_when_using_state() {
        implements_send::<State>();
    }

    #[test]
    const fn should_implement_sync_when_using_state() {
        implements_sync::<State>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_state() {
        implements_send::<State>();
        implements_sync::<State>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_state() {
        implements_sized::<State>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_state() {
        implements_hash::<State>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_state() {
        implements_partial_eq::<State>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_state() {
        implements_eq::<State>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_state() {
        implements_partial_ord::<State>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_state() {
        implements_ord::<State>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_state() {
        implements_debug::<State>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_state() {
        implements_clone::<State>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_state() {
        implements_unpin::<State>();
    }

    #[test]
    fn should_be_able_to_create_state_invalidinputdata_error_when_using_enum_directly() {
        let _result = State::InvalidInputData;
    }

    #[test]
    #[should_panic]
    fn should_not_be_able_to_create_state_invalidcikformat_error_when_passing_valid_cik_string() {
        // TODO: "InvalidCikFormat error should only be able to be created passing an invalid CIK format, it should not be able to be created by passing a valid CIK format.");
        let _result = State::InvalidCikFormat("123456789".to_string());
    }
}
