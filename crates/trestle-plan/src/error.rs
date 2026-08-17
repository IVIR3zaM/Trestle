//! The one type every validation failure in this crate produces.

use std::fmt;

/// One thing wrong with a plan or status file: the exact field that failed and
/// what it needed instead.
///
/// Under `D5` the agent writes plans, so this message is the interface it
/// converges against — `"invalid plan"` is a defect, `"units[3].oracle: required
/// when neither gate nor order is present"` is the bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError {
    /// A dotted/indexed path to the offending field, e.g. `units[3].oracle`.
    pub path: String,
    /// What was expected instead.
    pub message: String,
}

impl PlanError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        PlanError {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
