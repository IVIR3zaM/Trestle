//! T16: the privacy threat model (`docs/THREAT-MODEL.md`) turned into a test
//! suite. This crate ships no code of its own — its only deliverable is the
//! `egress` test module, so everything here lives behind `#[cfg(test)]` and a
//! non-test build of this crate is deliberately empty.
#![cfg(test)]

mod egress;
