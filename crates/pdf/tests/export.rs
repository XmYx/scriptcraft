//! PDF export of the sample script.

use scriptcraft_layout::{PageSetup, Paper, paginate};

const LANTERN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");

/// Count `/Type /Page` objects that are pages (not the page tree) in uncompressed output, or fall
/// back to counting `/MediaBox` entries.
fn page_count(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    text.matches("/MediaBox").count()
}

#[test]
fn sample_exports_one_pdf_page_per_layout_page() {
    let sp = scriptcraft_fountain::parse(LANTERN);
    for paper in Paper::ALL {
        let layout = paginate(&sp, &PageSetup { paper, scene_numbers: true, ..PageSetup::default() });
        let (_, font) = scriptcraft_fonts::pdf_face();
        let info = scriptcraft_pdf::Info { title: sp.title().map(str::to_owned), ..Default::default() };
        let pdf = scriptcraft_pdf::export(&layout, font, &info).expect("export");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 2_000);
        let pages = page_count(&pdf);
        assert_eq!(pages, layout.page_count() + 1, "{paper:?}: title page + script pages");
    }
}
