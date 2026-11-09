//! # Core Domain Types
//!
//! Provides the fact model, the standard concept set, and the types every source and every
//! taxonomy plug-in shares.
//!
//! The core knows no taxonomy and no source. Its types are open, so a new currency, form, or
//! namespace needs no change here.
//!
//! ## Modules
//!
//! - [`alias`]: Tiers 1 and 2 of resolution on one filing.
//! - [`candidate`]: The value one filing gives for one standard concept.
//! - [`chronology`]: The gap check over time.
//! - [`concept`]: Concept names, namespace families, and namespaces.
//! - [`confidence`]: How a standardized value was resolved.
//! - [`currency_codes`]: The tables of currency codes.
//! - [`dedup`]: The dedup rules of the standardized layer.
//! - [`elements`]: The standard concept set.
//! - [`entity_name`]: The name of a company.
//! - [`fact`]: The as-is fact, its value key, and the types of its fields.
//! - [`fact_set`]: The standardized facts of one entity and one period.
//! - [`filing`]: A filing, its ID, and the entity ID.
//! - [`finding`]: A fact about the data that a step reports without failing the run.
//! - [`fiscal_period`]: The fiscal period of a filing as filed.
//! - [`fiscal_year`]: The fiscal year of a filing as filed.
//! - [`form`]: The form type of a filing as filed.
//! - [`graph`]: The resolution and validation engine.
//! - [`invalid_value`]: The error of a core value type.
//! - [`period`]: Instants and durations.
//! - [`period_identity`]: The derivation of the fourth quarter from the fiscal year.
//! - [`release`]: The taxonomy release of a plug-in.
//! - [`rule`]: The ID of a rule.
//! - [`standardized`]: One value of one standard concept.
//! - [`suspicious_period`]: The check for periods that are most likely typos of the filer.
//! - [`taxonomy`]: The trait of a taxonomy plug-in, and the registry.
//! - [`unit`](mod@unit): Units and measures.

pub mod alias;
pub mod candidate;
pub mod chronology;
pub mod concept;
pub mod confidence;
pub mod currency_codes;
pub mod dedup;
pub mod elements;
pub mod entity_name;
pub mod fact;
pub mod fact_set;
pub mod filing;
pub mod finding;
pub mod fiscal_period;
pub mod fiscal_year;
pub mod form;
pub mod graph;
pub mod invalid_value;
pub mod period;
pub mod period_identity;
pub mod release;
pub mod rule;
pub mod standardized;
pub mod suspicious_period;
pub mod taxonomy;
pub mod unit;
