//! Spelling through craft-libs' `craft-proof`. English is always checked (an installed Hunspell
//! dictionary, else craft-proof's built-in word list); other languages need their dictionary.
//! Words in capitals (scene headings, character cues, shouted words) and words with digits are
//! taken as correct, as screenplays are full of them.

use std::ops::Range;

use scriptcraft_model::{ElementKind, Screenplay};

/// A misspelled word: element and character range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misspelling {
    pub element: usize,
    pub start: usize,
    pub len: usize,
    pub word: String,
}

/// Whether an element's text is checked: printed text that is not written in capitals.
pub fn checked(kind: ElementKind) -> bool {
    kind.is_printed() && !kind.is_uppercase()
}

/// Byte ranges of the misspelled words in `text`.
pub fn misspelled(text: &str) -> Vec<Range<usize>> {
    craft_proof::check_spelling_with(text, &craft_proof::Options::default())
        .into_iter()
        .filter(|i| i.kind == craft_proof::IssueKind::Spelling)
        .map(|i| i.start..i.end)
        .collect()
}

fn char_index(text: &str, byte: usize) -> usize {
    text.get(..byte).map_or(0, |s| s.chars().count())
}

/// Every misspelling in reading order.
pub fn all(sp: &Screenplay) -> Vec<Misspelling> {
    let mut out = Vec::new();
    for (element, e) in sp.elements.iter().enumerate().filter(|(_, e)| checked(e.kind)) {
        for r in misspelled(&e.text) {
            let word = e.text.get(r.clone()).unwrap_or("").to_owned();
            let start = char_index(&e.text, r.start);
            out.push(Misspelling { element, start, len: word.chars().count(), word });
        }
    }
    out
}

/// The first misspelling after (`element`, `char_index`), wrapping to the start.
pub fn next(sp: &Screenplay, element: usize, char_index: usize) -> Option<Misspelling> {
    let all = all(sp);
    all.iter().find(|m| (m.element, m.start) > (element, char_index)).or_else(|| all.first()).cloned()
}

/// Up to `max` suggestions for a word.
pub fn suggestions(word: &str, max: usize) -> Vec<String> {
    craft_proof::suggest(word, max)
}

/// Accept a word for the rest of the session (and every script).
pub fn add_word(word: &str) {
    craft_proof::add_word(word);
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::Element;

    #[test]
    fn finds_misspellings_but_not_capitals() {
        let sp = Screenplay {
            elements: vec![
                Element::new(ElementKind::SceneHeading, "INT. LIGTHOUSE - NIGHT"),
                Element::new(ElementKind::Action, "The lamp flikers in the wind."),
                Element::new(ElementKind::Dialogue, "Keep it BURNNING, please."),
            ],
            ..Screenplay::default()
        };
        let found = all(&sp);
        assert_eq!(found.iter().map(|m| m.word.as_str()).collect::<Vec<_>>(), ["flikers"]);
        assert_eq!((found[0].element, found[0].start, found[0].len), (1, 9, 7));
        assert!(suggestions("flikers", 5).iter().any(|s| s == "flickers"), "{:?}", suggestions("flikers", 5));
        assert_eq!(next(&sp, 1, 9), Some(found[0].clone()), "wraps around to the only one");
        add_word("flikers");
        assert!(all(&sp).is_empty());
    }

    #[test]
    fn multibyte_text_gives_character_offsets() {
        let sp = Screenplay { elements: vec![Element::new(ElementKind::Action, "Café señor wrongg.")], ..Screenplay::default() };
        let found = all(&sp);
        let m = found.iter().find(|m| m.word == "wrongg").expect("wrongg is flagged");
        assert_eq!(m.start, 11);
    }
}
