#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! PDF export: every page of a [`Layout`] drawn on its paper size with one embedded fixed-pitch
//! face, through craft-libs' `craft-pdf` (Krilla). The pages are exactly the layout's pages, so
//! the PDF breaks where the page preview, the plain-text export and the scene list say it does.
//!
//! Text sits on the grid: column × 7.2 pt across, 72 pt (one inch) + row × 12 pt down, in 12 pt
//! type. Page numbers print half an inch from the top; a revised page names its revision colour
//! at the top left, and revised lines carry `*` in the right margin.

use craft_pdf::write::{Metadata, Writer, backend};
use scriptcraft_layout::{Layout, Page, Role, TOP_MARGIN_IN};

/// Why the PDF was not written.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the screenplay font could not be embedded: {0}")]
    Font(String),
    #[error("{0}")]
    Write(#[from] craft_pdf::write::WriteError),
    #[error("the paper size is not valid")]
    Paper,
}

/// Points per inch.
const PT: f32 = 72.0;
const FONT_SIZE: f32 = 12.0;
/// Baseline below the top of a 12 pt row (the face's ascent, about 0.8 em).
const BASELINE: f32 = 9.6;

/// Document information.
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub title: Option<String>,
    pub author: Option<String>,
    /// The label for a revised page's header, by revision level (`"Blue Revision"`); the
    /// caller translates it.
    pub revision_label: Option<fn(u8) -> String>,
}

/// Write the layout as PDF bytes, embedding `font` (TrueType or OpenType bytes).
pub fn export(layout: &Layout, font: &[u8], info: &Info) -> Result<Vec<u8>, Error> {
    craft_pdf::write::guard(|| {
        let font = craft_pdf::write::font(font, 0, None).map_err(|e| Error::Font(e.to_string()))?;
        let metadata = Metadata {
            creator: "ScriptCraft".to_owned(),
            producer: format!("ScriptCraft {}", env!("CARGO_PKG_VERSION")),
            title: info.title.clone(),
            authors: info.author.iter().cloned().collect(),
            ..Metadata::default()
        };
        let mut writer = Writer::new(backend::SerializeSettings::default(), metadata);
        let (w_in, h_in) = layout.paper.size_in();
        let size = backend::geom::Size::from_wh(w_in * PT, h_in * PT).ok_or(Error::Paper)?;
        for page in layout.title_page.iter().chain(&layout.pages) {
            let mut pdf_page = writer.document_mut().start_page_with(backend::page::PageSettings::new(size));
            let mut surface = pdf_page.surface();
            draw_page(&mut surface, &font, page, info);
            surface.finish();
            pdf_page.finish();
        }
        Ok(writer.finish()?)
    })
}

fn draw_page(surface: &mut backend::surface::Surface<'_>, font: &backend::text::Font, page: &Page, info: &Info) {
    let mut text = |column: f32, top: f32, s: &str| {
        if s.is_empty() {
            return;
        }
        let point = backend::geom::Point::from_xy(column * PT / 10.0, top + BASELINE);
        surface.draw_text(point, font.clone(), FONT_SIZE, s, false, backend::text::TextDirection::LeftToRight);
    };
    let header_top = PT * 0.5;
    if let Some(header) = &page.header {
        text(75.0 - header.chars().count() as f32, header_top, header);
    }
    if page.revision > 0
        && let Some(label) = info.revision_label
    {
        text(15.0, header_top, &label(page.revision));
    }
    let body_top = TOP_MARGIN_IN * PT;
    for line in &page.lines {
        let top = body_top + line.row as f32 * FONT_SIZE;
        let _ = Role::RevisionMark;
        text(line.column as f32, top, &line.text);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_a_non_font() {
        let layout = scriptcraft_layout::paginate(&Default::default(), &Default::default());
        assert!(matches!(super::export(&layout, b"not a font", &super::Info::default()), Err(super::Error::Font(_)) | Err(super::Error::Write(_))));
    }
}
