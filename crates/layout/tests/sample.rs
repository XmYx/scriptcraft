//! Pagination of the bundled sample screenplay.

use scriptcraft_layout::{PageSetup, Paper, Role, paginate, render_text};

const LANTERN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");

#[test]
fn sample_paginates_on_both_papers() {
    let sp = scriptcraft_fountain::parse(LANTERN);
    for paper in Paper::ALL {
        let layout = paginate(&sp, &PageSetup { paper, scene_numbers: false });
        assert!(layout.page_count() >= 2, "{paper:?}: {} pages", layout.page_count());
        assert!(layout.title_page.is_some());
        let rows = paper.rows_per_page();
        for page in &layout.pages {
            for line in &page.lines {
                assert!(line.row < rows, "{paper:?} page {} row {}", page.number, line.row);
                assert!(line.column + line.text.chars().count() <= 80, "{line:?} runs past the right margin");
            }
        }
        // Every printed element has a start, and starts are in reading order.
        let starts: Vec<usize> = layout.starts.iter().flatten().map(|p| layout.absolute_row(*p)).collect();
        assert!(starts.windows(2).all(|w| w[0] < w[1]));
        let printed = sp.elements.iter().filter(|e| e.kind.is_printed()).count();
        assert_eq!(starts.len(), printed);
        // No page ends with a scene heading or a character cue.
        for page in &layout.pages {
            let last = page.lines.iter().rfind(|l| l.role != Role::SceneNumber);
            assert!(!matches!(
                last.map(|l| l.role),
                Some(Role::Element(scriptcraft_model::ElementKind::SceneHeading | scriptcraft_model::ElementKind::Character))
            ));
        }
    }
}

#[test]
fn plain_text_rendering() {
    let sp = scriptcraft_fountain::parse(LANTERN);
    let layout = paginate(&sp, &PageSetup::default());
    let text = render_text(&layout);
    assert_eq!(text.matches('\u{c}').count(), layout.page_count()); // title page + pages
    assert!(text.contains("               EXT. NORTH POINT LIGHTHOUSE - NIGHT"));
    assert!(text.contains("                                     MAYA\n"));
}
