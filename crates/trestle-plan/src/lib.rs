//! Parses, serialises and validates the plan format defined in
//! `docs/PLAN-FORMAT.md` and `schema/plan.schema.json`. Every later consumer
//! reads a plan through this crate, so its error messages are a product
//! surface, not internal notes (`AGENTS.md` §3): every failure names the
//! offending path and what was expected instead.
//!
//! Deliberately no I/O beyond what a caller hands it as a string — this crate
//! never opens a file itself, so it is testable without a repo or an agent.

mod decode;
mod deferred;
mod error;
mod oracle;
mod phase;
mod plan;
mod rule;
mod status;
mod unit;

pub use deferred::Deferred;
pub use error::PlanError;
pub use oracle::{Oracle, Provenance};
pub use phase::Phase;
pub use plan::{parse_plan, Plan};
pub use rule::Rule;
pub use status::{parse_status, validate_status, OracleResult, Override, Status, StatusRecord};
pub use unit::Unit;
