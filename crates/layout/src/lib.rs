#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! Screenplay formatting and pagination.
//!
//! The page is a grid of fixed-pitch cells: 10 characters per inch across and 6 lines per inch
//! down, the metrics of 12 pt Courier-style type. Every [`Line`] has a row in the page body and a
//! column counted from the left edge of the paper, so a renderer only has to multiply: column ×
//! 0.1 in, and 1 in + row × ⅙ in from the top.
//!
//! The rules (element indents, keeping headings and character cues with what follows, splitting
//! action and dialogue across pages with `(MORE)` and `(CONT'D)`) are the conventional ones for
//! spec screenplays; `docs/formatting.md` lists them with their measurements.

mod format;
mod paginate;
mod wrap;

pub use format::{Align, Format, format_of};
pub use paginate::{Layout, Line, Page, PageStart, Position, Role, paginate, render_text};
pub use wrap::wrap;

use serde::{Deserialize, Serialize};

/// Fixed-pitch characters per inch.
pub const CHARS_PER_INCH: f32 = 10.0;
/// Lines per inch.
pub const LINES_PER_INCH: f32 = 6.0;
/// Top and bottom margin of the body, in inches. The page number sits half an inch from the top.
pub const TOP_MARGIN_IN: f32 = 1.0;
pub const BOTTOM_MARGIN_IN: f32 = 1.0;

/// Paper sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Paper {
    #[default]
    UsLetter,
    A4,
}

impl Paper {
    pub const ALL: [Paper; 2] = [Paper::UsLetter, Paper::A4];

    /// Width and height in inches.
    pub fn size_in(self) -> (f32, f32) {
        match self {
            Paper::UsLetter => (8.5, 11.0),
            Paper::A4 => (210.0 / 25.4, 297.0 / 25.4),
        }
    }

    /// Body lines on a page: the height between the margins at six lines per inch.
    pub fn rows_per_page(self) -> usize {
        let (_, h) = self.size_in();
        // 54 on US Letter, 58 on A4; never below 10 whatever the constants become.
        (((h - TOP_MARGIN_IN - BOTTOM_MARGIN_IN) * LINES_PER_INCH).floor() as usize).max(10)
    }

    pub fn label(self) -> &'static str {
        match self {
            Paper::UsLetter => "US Letter",
            Paper::A4 => "A4",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Paper::UsLetter => "us-letter",
            Paper::A4 => "a4",
        }
    }

    pub fn from_id(id: &str) -> Option<Paper> {
        Paper::ALL.into_iter().find(|p| p.id().eq_ignore_ascii_case(id) || p.label().eq_ignore_ascii_case(id))
    }
}

/// How to lay out a script.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSetup {
    pub paper: Paper,
    /// Print scene numbers in both margins of each scene heading.
    pub scene_numbers: bool,
}
