//! Moving whole scenes (index cards).

use scriptcraft_model::Screenplay;

/// Move scene `from` (0-based, in script order) so it becomes scene `to`. A scene is its heading
/// and everything up to the next heading. Returns the new index of its heading.
pub fn move_scene(sp: &mut Screenplay, from: usize, to: usize) -> Option<usize> {
    let ranges = sp.scene_ranges();
    let range = ranges.get(from)?.clone();
    if from == to || to >= ranges.len() {
        return (from == to).then_some(range.start);
    }
    let block: Vec<_> = sp.elements.drain(range.clone()).collect();
    let len = block.len();
    let after = sp.scene_ranges();
    // After removal, scene `to` is at index `to` when moving up, `to - 1` + its end when moving down.
    let at = if to < from { after.get(to)?.start } else { after.get(to - 1).map_or(sp.elements.len(), |r| r.end) };
    let at = at.min(sp.elements.len());
    sp.elements.splice(at..at, block);
    let _ = len;
    Some(at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::{Element, ElementKind};

    fn script() -> Screenplay {
        let mut elements = vec![Element::new(ElementKind::Action, "FADE IN:")];
        for n in ["A", "B", "C"] {
            elements.push(Element::new(ElementKind::SceneHeading, format!("INT. {n} - DAY")));
            elements.push(Element::new(ElementKind::Action, format!("{n} happens.")));
        }
        Screenplay { elements, ..Screenplay::default() }
    }

    fn order(sp: &Screenplay) -> String {
        sp.scene_heading_indices().map(|h| sp.elements[h].text.chars().nth(5).unwrap_or('?')).collect()
    }

    #[test]
    fn moves_down_and_up_keeping_scene_bodies() {
        let mut sp = script();
        assert_eq!(move_scene(&mut sp, 0, 2), Some(5));
        assert_eq!(order(&sp), "BCA");
        assert_eq!(sp.elements[6].text, "A happens.");
        assert_eq!(move_scene(&mut sp, 2, 0), Some(1));
        assert_eq!(order(&sp), "ABC");
        assert_eq!(sp.elements[0].text, "FADE IN:");
        assert_eq!(move_scene(&mut sp, 1, 2), Some(5));
        assert_eq!(order(&sp), "ACB");
        assert_eq!(move_scene(&mut sp, 0, 9), None);
        assert_eq!(move_scene(&mut sp, 7, 0), None);
    }
}
