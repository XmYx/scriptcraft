//! Find and replace across the script's elements.

use scriptcraft_model::Screenplay;

/// One match: element, and character range in its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Match {
    pub element: usize,
    pub start: usize,
    pub len: usize,
}

/// Every match of `query` in reading order. Case-insensitive unless `match_case`. Matching is
/// per element (a match never spans two elements).
pub fn find_all(sp: &Screenplay, query: &str, match_case: bool) -> Vec<Match> {
    if query.is_empty() {
        return Vec::new();
    }
    let q: Vec<char> = if match_case { query.chars().collect() } else { query.chars().flat_map(char::to_lowercase).collect() };
    let mut out = Vec::new();
    for (element, e) in sp.elements.iter().enumerate() {
        // Compare char by char so offsets stay in characters of the original text.
        let text: Vec<char> = e.text.chars().collect();
        let fold = |c: char| -> char { if match_case { c } else { c.to_lowercase().next().unwrap_or(c) } };
        let mut i = 0;
        while i + q.len() <= text.len() {
            if text.get(i..i + q.len()).is_some_and(|w| w.iter().zip(&q).all(|(a, b)| fold(*a) == *b)) {
                out.push(Match { element, start: i, len: q.len() });
                i += q.len().max(1);
            } else {
                i += 1;
            }
        }
    }
    out
}

/// The first match at or after (`element`, `char_index`), wrapping to the start; `backward`
/// searches the other way from just before the position.
pub fn next(matches: &[Match], element: usize, char_index: usize, backward: bool) -> Option<Match> {
    if backward {
        matches.iter().rev().find(|m| (m.element, m.start) < (element, char_index)).or_else(|| matches.last()).copied()
    } else {
        matches.iter().find(|m| (m.element, m.start) >= (element, char_index)).or_else(|| matches.first()).copied()
    }
}

/// Replace one match. Returns false when the text no longer holds it.
pub fn replace_one(sp: &mut Screenplay, m: Match, with: &str) -> bool {
    let Some(e) = sp.elements.get_mut(m.element) else { return false };
    let chars: Vec<char> = e.text.chars().collect();
    if m.start + m.len > chars.len() {
        return false;
    }
    let mut text: String = chars.get(..m.start).unwrap_or_default().iter().collect();
    text.push_str(with);
    text.extend(chars.get(m.start + m.len..).unwrap_or_default());
    e.text = text;
    true
}

/// Replace every match; returns how many.
pub fn replace_all(sp: &mut Screenplay, query: &str, with: &str, match_case: bool) -> usize {
    let matches = find_all(sp, query, match_case);
    // From the end, so earlier offsets stay valid.
    for m in matches.iter().rev() {
        replace_one(sp, *m, with);
    }
    matches.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::{Element, ElementKind};

    fn sp() -> Screenplay {
        Screenplay {
            elements: vec![Element::new(ElementKind::Action, "The lamp. The LAMP!"), Element::new(ElementKind::Dialogue, "Lämp lamp")],
            ..Screenplay::default()
        }
    }

    #[test]
    fn finds_without_and_with_case() {
        let s = sp();
        assert_eq!(find_all(&s, "lamp", false).len(), 3);
        assert_eq!(find_all(&s, "lamp", true).len(), 2);
        assert_eq!(find_all(&s, "lämp", false), [Match { element: 1, start: 0, len: 4 }]);
        assert!(find_all(&s, "", false).is_empty());
    }

    #[test]
    fn next_wraps_both_ways() {
        let m = find_all(&sp(), "lamp", false);
        assert_eq!(next(&m, 0, 5, false), Some(m[1]));
        assert_eq!(next(&m, 1, 9, false), Some(m[0]));
        assert_eq!(next(&m, 0, 0, true), m.last().copied());
    }

    #[test]
    fn replaces_keeping_multibyte_text_intact() {
        let mut s = sp();
        assert_eq!(replace_all(&mut s, "lamp", "light", false), 3);
        assert_eq!(s.elements[0].text, "The light. The light!");
        assert_eq!(s.elements[1].text, "Lämp light");
    }
}
