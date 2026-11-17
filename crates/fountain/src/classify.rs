//! How the first line of a paragraph decides what the paragraph is. The reader and the writer
//! share this, so the writer can tell when an element needs a forcing mark.

/// What a paragraph reads as, from its first line (trimmed) and its line count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Natural {
    PageBreak,
    Section,
    Synopsis,
    Lyric,
    Note,
    Boneyard,
    ForcedAction,
    ForcedCharacter,
    ForcedHeading,
    Centered,
    ForcedTransition,
    Heading,
    Transition,
    Character,
    Shot,
    Action,
}

const HEADING_STARTS: [&str; 6] = ["INT./EXT", "INT/EXT", "I/E", "INT", "EXT", "EST"];

/// Whether `line` starts like a scene heading: `INT`, `EXT`, `EST`, `INT./EXT`, `INT/EXT` or `I/E`,
/// in any case, followed by `.` or a space.
pub fn is_scene_heading_start(line: &str) -> bool {
    let upper = line.trim_start().to_uppercase();
    HEADING_STARTS.iter().any(|start| upper.strip_prefix(start).is_some_and(|rest| rest.starts_with('.') || rest.starts_with(' ')))
}

/// Words a shot starts with. Shots have no mark of their own in Fountain; an all-capitals line on
/// its own that starts with one of these reads as a shot.
const SHOT_STARTS: [&str; 22] = [
    "ANGLE ON",
    "CLOSE ON",
    "CLOSE UP",
    "CLOSE-UP",
    "CLOSEUP",
    "EXTREME CLOSE",
    "WIDE ON",
    "WIDE SHOT",
    "MEDIUM SHOT",
    "TWO SHOT",
    "INSERT",
    "POV",
    "REVERSE ANGLE",
    "OVER THE SHOULDER",
    "BACK TO SCENE",
    "ESTABLISHING",
    "AERIAL",
    "TRACKING",
    "OVERHEAD",
    "LOW ANGLE",
    "HIGH ANGLE",
    "PUSH IN",
];

/// Whether an all-capitals `line` starts with a shot word (`CLOSE ON`, `ANGLE ON`, `POV`…).
pub fn is_shot_start(line: &str) -> bool {
    let line = line.trim();
    is_capitals(line)
        && SHOT_STARTS
            .iter()
            .any(|start| line.strip_prefix(start).is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', ':', '-', '.', '\''])))
}

/// At least one letter and no lowercase letters.
pub(crate) fn is_capitals(s: &str) -> bool {
    s.chars().any(char::is_alphabetic) && !s.chars().any(char::is_lowercase)
}

/// Whether `cue` is a character cue without a forcing mark: the name (before any parenthesis)
/// in capitals. A trailing `^` (dual dialogue) is allowed.
pub(crate) fn is_natural_cue(cue: &str) -> bool {
    let cue = cue.trim().trim_end_matches('^').trim_end();
    let name = cue.split('(').next().unwrap_or(cue);
    is_capitals(name)
}

fn is_page_break(line: &str) -> bool {
    line.len() >= 3 && line.chars().all(|c| c == '=')
}

/// Classify a paragraph by its first line (already trimmed) and its number of lines. `whole` is
/// the paragraph's text, used to tell a note paragraph from action that starts with a note.
pub(crate) fn classify(first: &str, lines: usize, whole: &str) -> Natural {
    if is_page_break(first) {
        return Natural::PageBreak;
    }
    if first.starts_with('#') {
        return Natural::Section;
    }
    if first.starts_with('=') {
        return Natural::Synopsis;
    }
    if first.starts_with('~') {
        return Natural::Lyric;
    }
    if first.starts_with("/*") {
        return Natural::Boneyard;
    }
    if first.starts_with('!') {
        return Natural::ForcedAction;
    }
    if first.starts_with('@') {
        return Natural::ForcedCharacter;
    }
    if first.starts_with('.') && !first.starts_with("..") {
        return Natural::ForcedHeading;
    }
    if first.starts_with('>') {
        return if first.ends_with('<') { Natural::Centered } else { Natural::ForcedTransition };
    }
    if is_note_paragraph(whole) {
        return Natural::Note;
    }
    if lines == 1 && is_scene_heading_start(first) {
        return Natural::Heading;
    }
    if lines == 1 && is_capitals(first) && first.ends_with("TO:") {
        return Natural::Transition;
    }
    if lines >= 2 && is_natural_cue(first) && !is_scene_heading_start(first) {
        return Natural::Character;
    }
    if lines == 1 && is_shot_start(first) {
        return Natural::Shot;
    }
    Natural::Action
}

/// A paragraph that is one `[[note]]` and nothing else.
pub(crate) fn is_note_paragraph(whole: &str) -> bool {
    let t = whole.trim();
    t.len() >= 4
        && t.starts_with("[[")
        && t.ends_with("]]")
        && t.get(2..t.len() - 2).is_some_and(|inner| !inner.contains("]]") && !inner.contains("[["))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_starts() {
        assert!(is_scene_heading_start("INT. KITCHEN - NIGHT"));
        assert!(is_scene_heading_start("ext. roof - day"));
        assert!(is_scene_heading_start("INT./EXT. CAR - MOVING"));
        assert!(is_scene_heading_start("I/E VAN"));
        assert!(!is_scene_heading_start("INTERIOR DESIGN"));
        assert!(!is_scene_heading_start("EXTRA"));
    }

    #[test]
    fn shot_starts() {
        assert!(is_shot_start("CLOSE ON THE LANTERN"));
        assert!(is_shot_start("POV"));
        assert!(!is_shot_start("Close on the lantern"));
        assert!(!is_shot_start("POVERTY"));
    }

    #[test]
    fn classify_basics() {
        assert_eq!(classify("CUT TO:", 1, "CUT TO:"), Natural::Transition);
        assert_eq!(classify("MAYA", 2, "MAYA\nHi."), Natural::Character);
        assert_eq!(classify("MAYA", 1, "MAYA"), Natural::Action);
        assert_eq!(classify("INT. A - DAY", 2, ""), Natural::Action);
        assert_eq!(classify("[[a note]]", 1, "[[a note]]"), Natural::Note);
        assert_eq!(classify("[[a]] and [[b]]", 1, "[[a]] and [[b]]"), Natural::Action);
        assert_eq!(classify("...and then", 1, "...and then"), Natural::Action);
        assert_eq!(classify("> THE END <", 1, ""), Natural::Centered);
    }
}
