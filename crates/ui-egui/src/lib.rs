#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! ScriptCraft's egui interface.
//!
//! [`ScriptApp`] owns the open [`scriptcraft_engine::Document`] and draws the menu bar, the
//! toolbar, the scene navigator, the script (as an editable draft on paper, or as the exact
//! printed pages) and the status bar. Every menu item runs a [`scriptcraft_engine::Command`].

mod app;
mod cards;
mod dialogs;
mod editor;
mod find;
mod fonts;
pub mod i18n;
mod keys;
mod navigator;
mod preview;
mod reports;

pub use app::{SAMPLE_FOUNTAIN, ScriptApp, Startup, View};
pub use fonts::install as install_fonts;
pub use scriptcraft_engine::Command;
