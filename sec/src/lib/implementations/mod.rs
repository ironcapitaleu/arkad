//! # Implementations
//!
//! Provides the concrete state machine that processes SEC filings, built on the abstractions in
//! [`crate::traits`].
//!
//! Where [`crate::traits`] defines *what* a state, context, and transition are, this module
//! supplies the *real* ETL pipeline: the individual states and the transitions that wire them
//! together. The two concerns are split into separate child modules so a state's logic stays
//! independent of how it connects to its neighbours.
//!
//! ## Modules
//!
//! - [`states`]: The Extract, Transform, and Load states, each with its own input, output, and context.
//! - [`transitions`]: The `TryFrom`/`From` conversions that move one state's output into the next state.
//!
//! ## See Also
//!
//! - [`crate::traits`]: The state-machine traits these types implement.
//! - [`crate::shared`]: The shared domain types (e.g. [`Cik`](crate::shared::cik::Cik)) the states operate on.

pub mod states;
pub mod transitions;
