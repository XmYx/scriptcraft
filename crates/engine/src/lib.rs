#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! The ScriptCraft editing engine: everything the editor does that is not drawing.
//!
//! - [`Document`]: the open script, its file, page setup, undo and a cached layout.
//! - [`typing`]: what Enter, Tab and Backspace do to elements.
//! - [`complete`]: auto-complete for character names, scene headings and transitions.
//! - [`Command`]: every menu action by id, for the UI, the command-line tool and agents.
//! - [`io`]: opening Fountain and FDX, saving Fountain atomically, exporting.

pub mod autosave;
pub mod command;
pub mod complete;
mod document;
pub mod find;
pub mod io;
pub mod numbering;
pub mod reports;
pub mod revisions;
pub mod scenes;
pub mod spelling;
pub mod typing;

pub use command::Command;
pub use document::Document;
pub use io::EngineError;

/// A random 16-hex-digit id for a scene (`[[scene-id: …]]`). Not cryptographic: it only has to
/// differ between scenes and scripts.
pub fn new_scene_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(nanos);
    format!("{:016x}", h.finish())
}
