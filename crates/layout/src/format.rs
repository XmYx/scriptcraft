//! Where each element sits on the page.

use scriptcraft_model::ElementKind;

/// Horizontal alignment inside an element's column range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// The column range and spacing of one element kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    /// First column, from the left edge of the paper, in characters (tenths of an inch).
    pub left: usize,
    /// Columns available for text.
    pub width: usize,
    pub align: Align,
    /// Blank lines before the element (dropped at the top of a page).
    pub space_before: usize,
}

/// The format of a printed kind; `None` for kinds that are not printed.
pub fn format_of(kind: ElementKind) -> Option<Format> {
    let f = |left, width, align, space_before| Some(Format { left, width, align, space_before });
    match kind {
        ElementKind::SceneHeading | ElementKind::Shot => f(15, 60, Align::Left, 1),
        ElementKind::Action => f(15, 60, Align::Left, 1),
        ElementKind::Character => f(37, 38, Align::Left, 1),
        ElementKind::Parenthetical => f(31, 25, Align::Left, 0),
        ElementKind::Dialogue | ElementKind::Lyric => f(25, 35, Align::Left, 0),
        ElementKind::Transition => f(15, 60, Align::Right, 1),
        ElementKind::Centered => f(15, 60, Align::Center, 1),
        ElementKind::Section | ElementKind::Synopsis | ElementKind::Note | ElementKind::Boneyard | ElementKind::PageBreak => None,
    }
}
