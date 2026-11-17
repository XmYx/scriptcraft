#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! Fountain (`.fountain`), the plain-text screenplay format, read into and written from
//! [`scriptcraft_model::Screenplay`].
//!
//! Written from the public Fountain syntax description (fountain.io, "Syntax", version 1.1).
//! Reading never fails on text: anything unrecognised is action. [`parse_bytes`] adds the size
//! limit and UTF-8 handling for files.
//!
//! The writer prefers the natural form of each element and falls back to Fountain's forcing marks
//! (`.` `!` `@` `>`) whenever the natural form would read back as something else, so
//! `parse(&write(&s)) == s` for every screenplay that [`parse`] produced. See `docs/fountain.md`
//! for the few model states Fountain cannot carry (shots that don't start with a shot word,
//! dialogue without a character).

mod classify;
mod read;
mod write;

pub use classify::{is_scene_heading_start, is_shot_start};
pub use read::{parse, parse_bytes};
pub use write::write;

/// The largest Fountain file [`parse_bytes`] reads: 64 MiB, far beyond any real script.
pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// Why a Fountain file was not read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("the file is {0} bytes; Fountain files over {MAX_INPUT_BYTES} bytes are not opened")]
    TooLarge(usize),
}
