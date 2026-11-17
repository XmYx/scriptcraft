//! Copying, cutting and pasting whole elements as Fountain text, so screenplay text moves
//! between ScriptCraft windows and other screenwriting apps with its element types intact.

use std::ops::Range;

use scriptcraft_model::{Element, ElementKind, Screenplay};

/// The elements in `range` as Fountain text (no title page).
pub fn copy(sp: &Screenplay, range: Range<usize>) -> String {
    let elements = sp.elements.get(range).unwrap_or_default().to_vec();
    scriptcraft_fountain::write(&Screenplay { elements, ..Screenplay::default() })
}

/// Whether pasted text should become elements: more than one paragraph. A single line is pasted
/// into the current element as text.
pub fn is_screenplay(text: &str) -> bool {
    let t = text.replace("\r\n", "\n");
    t.trim().contains("\n\n")
}

/// Paste Fountain (or plain screenplay text) as elements after element `at`, or in its place when
/// it is empty. A title page in the text is ignored. Returns the range of the new elements.
pub fn paste(sp: &mut Screenplay, at: usize, text: &str) -> Option<Range<usize>> {
    let pasted = scriptcraft_fountain::parse(text).elements;
    if pasted.is_empty() {
        return None;
    }
    let n = pasted.len();
    let replace = sp.elements.get(at).is_some_and(|e| e.text.trim().is_empty() && e.kind != ElementKind::PageBreak);
    let start = if replace { at } else { (at + 1).min(sp.elements.len()) };
    let end_old = if replace { at + 1 } else { start };
    sp.elements.splice(start..end_old.min(sp.elements.len()), pasted);
    Some(start..start + n)
}

/// Take out the elements in `range`, keeping at least one (empty action) element.
pub fn delete(sp: &mut Screenplay, range: Range<usize>) -> usize {
    let end = range.end.min(sp.elements.len());
    let start = range.start.min(end);
    sp.elements.drain(start..end);
    if sp.elements.is_empty() {
        sp.elements.push(Element::new(ElementKind::Action, ""));
    }
    start.min(sp.elements.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(sp: &Screenplay) -> Vec<ElementKind> {
        sp.elements.iter().map(|e| e.kind).collect()
    }

    #[test]
    fn copy_then_paste_keeps_element_types() {
        let src = scriptcraft_fountain::parse("INT. A - DAY\n\nRain.\n\nMAYA\n(quiet)\nGo.\n\nCUT TO:\n");
        let text = copy(&src, 1..5);
        assert_eq!(text, "Rain.\n\nMAYA\n(quiet)\nGo.\n");
        let mut dst = scriptcraft_fountain::parse("INT. B - NIGHT\n\nWind.\n");
        assert_eq!(paste(&mut dst, 0, &text), Some(1..5));
        assert_eq!(
            kinds(&dst),
            [
                ElementKind::SceneHeading,
                ElementKind::Action,
                ElementKind::Character,
                ElementKind::Parenthetical,
                ElementKind::Dialogue,
                ElementKind::Action
            ]
        );
    }

    #[test]
    fn paste_replaces_an_empty_element_and_ignores_title_pages() {
        let mut sp = Screenplay { elements: vec![Element::new(ElementKind::SceneHeading, "")], ..Screenplay::default() };
        assert_eq!(paste(&mut sp, 0, "Title: X\n\nEXT. ROOF - DAWN\n\nShe waits.\n"), Some(0..2));
        assert_eq!(kinds(&sp), [ElementKind::SceneHeading, ElementKind::Action]);
        assert!(sp.title_page.is_empty());
    }

    #[test]
    fn single_paragraphs_are_text_and_delete_keeps_one_element() {
        assert!(!is_screenplay("just a line\n"));
        assert!(is_screenplay("INT. A - DAY\r\n\r\nRain.\r\n"));
        let mut sp = scriptcraft_fountain::parse("A.\n\nB.\n\nC.\n");
        assert_eq!(delete(&mut sp, 1..3), 0, "the caret goes to the element before, the only one left");
        assert_eq!(sp.elements.len(), 1);
        assert_eq!(delete(&mut sp, 0..9), 0);
        assert_eq!(sp.elements.len(), 1);
        assert_eq!(sp.elements[0].text, "");
    }
}
