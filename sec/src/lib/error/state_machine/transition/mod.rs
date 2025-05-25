#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fmt::Debug, hash::Hash};
    
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_transition() {
        implements_auto_traits::<Transition>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implementend_send_when_using_transition() {
        implements_send::<Transition>();
    }

    #[test]
    const fn should_implement_sync_when_using_transition() {
        implements_sync::<Transition>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_transition() {
        implements_send::<Transition>();
        implements_sync::<Transition>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_transition() {
        implements_sized::<Transition>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_transition() {
        implements_hash::<Transition>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_transition() {
        implements_partial_eq::<Transition>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_transition() {
        implements_eq::<Transition>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_transition() {
        implements_partial_ord::<Transition>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_transition() {
        implements_ord::<Transition>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_transition() {
        implements_debug::<Transition>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_transition() {
        implements_clone::<Transition>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_transition() {
        implements_unpin::<Transition>();
    }

    #[test]
    fn should_be_able_to_create_transition_failedcontextconversion_error_when_using_enum_directly()
    {
        let _result = Transition::FailedContextConversion;
    }
}
