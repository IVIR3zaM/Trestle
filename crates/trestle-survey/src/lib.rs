//! Repo survey + code-graph extraction (T05). Reads a repository on disk and
//! produces a structured, versioned picture of it — languages, module
//! boundaries, import edges, discovered test/build commands, and the shape
//! signals T03 scores. Deterministic analysis only: no model call, no
//! network, no writes to the repository being surveyed (`D0`).

mod ci;
mod co_change;
mod commands;
mod conventions;
mod language;
mod modules;
mod regex_imports;
mod repo_files;
mod schema;
mod shape_signals;
mod survey;
mod tree_sitter_imports;

pub use schema::Survey;
pub use survey::survey;
