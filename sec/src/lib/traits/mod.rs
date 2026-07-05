//! # Traits
//!
//! Provides the trait contracts the SEC state machine is built on.
//!
//! These traits define the abstract shape of states, transitions, context, data, and error
//! conversion; the concrete types in [`crate::implementations`] fill them in. Keeping the contracts
//! here lets the pipeline depend on behavior rather than concrete types.
//!
//! ## Modules
//!
//! - [`state_machine`]: The state machine, state, super-state, and transition traits.
//! - [`error`]: The [`FromDomainError`](error::FromDomainError) conversion trait.
//!
//! ## See Also
//!
//! - [`crate::implementations`]: Concrete types implementing these traits.

pub mod state_machine;

pub mod error;
