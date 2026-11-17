//! Auto-complete while typing: character names, scene headings and transitions.

use std::collections::HashMap;

use scriptcraft_model::{ElementKind, Screenplay, character_name};

/// The most suggestions offered at once.
pub const MAX_SUGGESTIONS: usize = 8;

/// Every character name in the script, most frequent first (ties alphabetical). `skip` leaves
/// out one element, the one being typed.
pub fn character_names(sp: &Screenplay, skip: Option<usize>) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (i, e) in sp.elements.iter().enumerate() {
        if Some(i) == skip {
            continue;
        }
        if let Some(name) = e.character_name() {
            *counts.entry(name).or_default() += 1;
        }
    }
    let mut names: Vec<(String, usize)> = counts.into_iter().collect();
    names.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    names.into_iter().map(|(n, _)| n).collect()
}

const HEADING_PREFIXES: [&str; 5] = ["INT. ", "EXT. ", "INT./EXT. ", "EST. ", "I/E "];
const TIMES_OF_DAY: [&str; 10] = ["DAY", "NIGHT", "CONTINUOUS", "LATER", "MORNING", "AFTERNOON", "EVENING", "DAWN", "DUSK", "MOMENTS LATER"];
const TRANSITIONS: [&str; 9] =
    ["CUT TO:", "DISSOLVE TO:", "MATCH CUT TO:", "SMASH CUT TO:", "JUMP CUT TO:", "FADE IN:", "FADE OUT.", "FADE TO BLACK.", "TIME CUT:"];

/// Suggestions for element `index` given its current text: whole replacement texts, best first.
/// Empty when nothing fits or the text already is the only suggestion.
pub fn suggestions(sp: &Screenplay, index: usize) -> Vec<String> {
    let Some(e) = sp.elements.get(index) else { return Vec::new() };
    let typed = e.text.trim_start().to_uppercase();
    let list = match e.kind {
        ElementKind::Character => {
            if typed.is_empty() || typed.contains('(') {
                return Vec::new();
            }
            filter_prefix(character_names(sp, Some(index)), &typed)
        }
        ElementKind::SceneHeading => heading_suggestions(sp, index, &typed),
        ElementKind::Transition => {
            if typed.is_empty() {
                return Vec::new();
            }
            filter_prefix(TRANSITIONS.iter().map(|t| (*t).to_owned()).collect(), &typed)
        }
        _ => Vec::new(),
    };
    let mut list: Vec<String> = list.into_iter().filter(|s| *s != typed && *s != character_name(&typed)).collect();
    list.truncate(MAX_SUGGESTIONS);
    list
}

fn filter_prefix(candidates: Vec<String>, typed: &str) -> Vec<String> {
    candidates.into_iter().filter(|c| c.starts_with(typed)).collect()
}

fn heading_suggestions(sp: &Screenplay, index: usize, typed: &str) -> Vec<String> {
    if typed.is_empty() {
        return Vec::new();
    }
    // "INT. LIGHTHOUSE - " → times of day.
    if let Some((place, time)) = typed.rsplit_once(" - ") {
        return TIMES_OF_DAY.iter().filter(|t| t.starts_with(time.trim())).map(|t| format!("{place} - {t}")).collect();
    }
    let has_prefix = HEADING_PREFIXES.iter().any(|p| typed.starts_with(p.trim_end()) && typed.len() > p.trim_end().len());
    if !has_prefix {
        return HEADING_PREFIXES.iter().filter(|p| p.starts_with(typed)).map(|p| (*p).to_owned()).collect();
    }
    // Places already used, without their time of day, most recent first.
    let mut places: Vec<String> = Vec::new();
    for (i, e) in sp.elements.iter().enumerate().rev() {
        if i == index || e.kind != ElementKind::SceneHeading {
            continue;
        }
        let upper = e.text.trim().to_uppercase();
        let place = upper.rsplit_once(" - ").map_or(upper.as_str(), |(p, _)| p).to_owned();
        if place.starts_with(typed) && !places.contains(&place) {
            places.push(place);
        }
    }
    places.into_iter().map(|p| format!("{p} - ")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::Element;

    fn sp(elements: &[(ElementKind, &str)]) -> Screenplay {
        Screenplay { elements: elements.iter().map(|(k, t)| Element::new(*k, *t)).collect(), ..Screenplay::default() }
    }

    #[test]
    fn character_names_by_frequency() {
        let s = sp(&[
            (ElementKind::Character, "TOMAS"),
            (ElementKind::Character, "MAYA"),
            (ElementKind::Character, "Maya (V.O.)"),
            (ElementKind::Character, "MA"),
        ]);
        assert_eq!(character_names(&s, Some(3)), ["MAYA", "TOMAS"]);
        assert_eq!(suggestions(&s, 3), ["MAYA"]);
    }

    #[test]
    fn no_suggestion_for_a_complete_name() {
        let s = sp(&[(ElementKind::Character, "MAYA"), (ElementKind::Character, "maya")]);
        assert!(suggestions(&s, 1).is_empty());
    }

    #[test]
    fn heading_prefixes_places_and_times() {
        let s = sp(&[(ElementKind::SceneHeading, "INT. LIGHTHOUSE - NIGHT"), (ElementKind::SceneHeading, "i")]);
        assert_eq!(suggestions(&s, 1), ["INT. ", "INT./EXT. ", "I/E "]);
        let s = sp(&[(ElementKind::SceneHeading, "INT. LIGHTHOUSE - NIGHT"), (ElementKind::SceneHeading, "int. li")]);
        assert_eq!(suggestions(&s, 1), ["INT. LIGHTHOUSE - "]);
        let s = sp(&[(ElementKind::SceneHeading, "INT. LIGHTHOUSE - d")]);
        assert_eq!(suggestions(&s, 0), ["INT. LIGHTHOUSE - DAY", "INT. LIGHTHOUSE - DAWN", "INT. LIGHTHOUSE - DUSK"]);
    }

    #[test]
    fn transitions() {
        let s = sp(&[(ElementKind::Transition, "d")]);
        assert_eq!(suggestions(&s, 0), ["DISSOLVE TO:"]);
    }
}
