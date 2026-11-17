//! What Enter, Tab and Backspace do. The rules are ScriptCraft's own, written down in
//! `docs/keyboard.md`:
//!
//! - **Enter** ends the element and starts the one that usually follows: scene heading → action,
//!   character → dialogue, parenthetical → dialogue, dialogue → action, transition → scene
//!   heading, everything else → action. Enter in the middle of an element splits it there.
//! - **Tab** changes the current element's kind: action → character → transition → scene heading
//!   → shot → action, and dialogue ⇄ parenthetical. **Shift+Tab** goes the other way.
//! - **Backspace** at the start of an element joins it to the previous one (an empty element is
//!   taken out).

use scriptcraft_model::{Element, ElementKind, Screenplay};

/// The kind Enter starts after an element of `kind`.
pub fn after_enter(kind: ElementKind) -> ElementKind {
    match kind {
        ElementKind::Character | ElementKind::Parenthetical => ElementKind::Dialogue,
        ElementKind::Transition => ElementKind::SceneHeading,
        ElementKind::Lyric => ElementKind::Lyric,
        _ => ElementKind::Action,
    }
}

const TAB_CYCLE: [ElementKind; 5] =
    [ElementKind::Action, ElementKind::Character, ElementKind::Transition, ElementKind::SceneHeading, ElementKind::Shot];

/// The kind Tab (forward) or Shift+Tab (backward) turns `kind` into.
pub fn on_tab(kind: ElementKind, forward: bool) -> ElementKind {
    match kind {
        ElementKind::Dialogue => ElementKind::Parenthetical,
        ElementKind::Parenthetical => ElementKind::Dialogue,
        k => match TAB_CYCLE.iter().position(|c| *c == k) {
            Some(i) => {
                let n = TAB_CYCLE.len();
                let next = if forward { (i + 1) % n } else { (i + n - 1) % n };
                TAB_CYCLE.get(next).copied().unwrap_or(ElementKind::Action)
            }
            None => ElementKind::Action,
        },
    }
}

/// Byte offset of character `char_index` in `s` (the end when past it).
pub fn byte_offset(s: &str, char_index: usize) -> usize {
    s.char_indices().nth(char_index).map_or(s.len(), |(b, _)| b)
}

/// Where the caret goes after an edit: element and character offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caret {
    pub element: usize,
    pub char_index: usize,
}

/// Change element `index` to `kind`. A parenthetical gets its parentheses; leaving one drops
/// empty ones. Returns where the caret goes.
pub fn set_kind(sp: &mut Screenplay, index: usize, kind: ElementKind) -> Option<Caret> {
    let e = sp.elements.get_mut(index)?;
    if e.kind == ElementKind::Parenthetical && kind != ElementKind::Parenthetical && e.text.trim() == "()" {
        e.text.clear();
    }
    e.set_kind(kind);
    if kind == ElementKind::Parenthetical && e.text.trim().is_empty() {
        e.text = "()".to_owned();
        return Some(Caret { element: index, char_index: 1 });
    }
    Some(Caret { element: index, char_index: e.text.chars().count() })
}

/// Enter at character `char_index` of element `index`.
pub fn enter(sp: &mut Screenplay, index: usize, char_index: usize) -> Option<Caret> {
    let e = sp.elements.get(index)?;
    let kind = e.kind;
    let len = e.text.chars().count();
    if char_index == 0 && len > 0 {
        // At the start of a non-empty element: open an empty element of the same kind above it.
        sp.elements.insert(index, Element::new(kind, ""));
        return Some(Caret { element: index + 1, char_index: 0 });
    }
    let at = byte_offset(&e.text, char_index.min(len));
    let e = sp.elements.get_mut(index)?;
    let tail = e.text.split_off(at);
    let kept = e.text.trim_end().len();
    e.text.truncate(kept);
    let next_kind = after_enter(kind);
    let mut next = Element::new(next_kind, tail.trim_start());
    if next_kind == ElementKind::Parenthetical && next.text.is_empty() {
        next.text = "()".to_owned();
    }
    sp.elements.insert(index + 1, next);
    Some(Caret { element: index + 1, char_index: 0 })
}

/// Backspace at the very start of element `index`: take it out if empty, otherwise join it to
/// the previous element. `None` when there is nothing before it.
pub fn backspace_at_start(sp: &mut Screenplay, index: usize) -> Option<Caret> {
    if index == 0 || index >= sp.elements.len() {
        return None;
    }
    let joined = sp.elements.remove(index);
    let previous = sp.elements.get_mut(index - 1)?;
    let char_index = previous.text.chars().count();
    previous.text.push_str(&joined.text);
    Some(Caret { element: index - 1, char_index })
}

/// Insert an element after `index` (or at the end) and return its index.
pub fn insert_after(sp: &mut Screenplay, index: Option<usize>, element: Element) -> usize {
    let at = index.map_or(sp.elements.len(), |i| (i + 1).min(sp.elements.len()));
    sp.elements.insert(at, element);
    at
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(elements: &[(ElementKind, &str)]) -> Screenplay {
        Screenplay { elements: elements.iter().map(|(k, t)| Element::new(*k, *t)).collect(), ..Screenplay::default() }
    }

    #[test]
    fn enter_follows_the_usual_order() {
        assert_eq!(after_enter(ElementKind::SceneHeading), ElementKind::Action);
        assert_eq!(after_enter(ElementKind::Character), ElementKind::Dialogue);
        assert_eq!(after_enter(ElementKind::Dialogue), ElementKind::Action);
        assert_eq!(after_enter(ElementKind::Transition), ElementKind::SceneHeading);
    }

    #[test]
    fn tab_cycles_both_ways() {
        let mut k = ElementKind::Action;
        for _ in 0..TAB_CYCLE.len() {
            let next = on_tab(k, true);
            assert_eq!(on_tab(next, false), k);
            k = next;
        }
        assert_eq!(k, ElementKind::Action);
        assert_eq!(on_tab(ElementKind::Dialogue, true), ElementKind::Parenthetical);
        assert_eq!(on_tab(ElementKind::Parenthetical, false), ElementKind::Dialogue);
    }

    #[test]
    fn enter_splits_and_opens_the_next_kind() {
        let mut s = sp(&[(ElementKind::Character, "MAYA")]);
        let caret = enter(&mut s, 0, 4).unwrap();
        assert_eq!(caret, Caret { element: 1, char_index: 0 });
        assert_eq!(s.elements[1].kind, ElementKind::Dialogue);
        let mut s = sp(&[(ElementKind::Action, "One. Two.")]);
        enter(&mut s, 0, 5).unwrap();
        assert_eq!((s.elements[0].text.as_str(), s.elements[1].text.as_str()), ("One.", "Two."));
    }

    #[test]
    fn enter_at_start_opens_a_line_above() {
        let mut s = sp(&[(ElementKind::Action, "Text")]);
        assert_eq!(enter(&mut s, 0, 0), Some(Caret { element: 1, char_index: 0 }));
        assert_eq!(s.elements[0].text, "");
        assert_eq!(s.elements[1].text, "Text");
    }

    #[test]
    fn enter_past_the_end_and_on_multibyte_text_is_safe() {
        let mut s = sp(&[(ElementKind::Action, "héllo wörld")]);
        enter(&mut s, 0, 99).unwrap();
        assert_eq!(s.elements[0].text, "héllo wörld");
        enter(&mut s, 0, 2).unwrap();
        assert_eq!((s.elements[0].text.as_str(), s.elements[1].text.as_str()), ("hé", "llo wörld"));
        assert_eq!(enter(&mut s, 42, 0), None);
    }

    #[test]
    fn backspace_joins_or_takes_out() {
        let mut s = sp(&[(ElementKind::Action, "One"), (ElementKind::Action, ""), (ElementKind::Action, " two")]);
        assert_eq!(backspace_at_start(&mut s, 1), Some(Caret { element: 0, char_index: 3 }));
        assert_eq!(backspace_at_start(&mut s, 1), Some(Caret { element: 0, char_index: 3 }));
        assert_eq!(s.elements[0].text, "One two");
        assert_eq!(backspace_at_start(&mut s, 0), None);
    }

    #[test]
    fn parenthetical_gets_and_drops_parentheses() {
        let mut s = sp(&[(ElementKind::Dialogue, "")]);
        assert_eq!(set_kind(&mut s, 0, ElementKind::Parenthetical), Some(Caret { element: 0, char_index: 1 }));
        assert_eq!(s.elements[0].text, "()");
        set_kind(&mut s, 0, ElementKind::Dialogue);
        assert_eq!(s.elements[0].text, "");
    }
}
