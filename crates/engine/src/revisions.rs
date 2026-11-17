//! Revision marks kept with a Fountain file. Fountain has no revision syntax, so ScriptCraft
//! stores them in an omitted-text block at the end of the file, which every Fountain reader
//! keeps and never prints:
//!
//! ```text
//! /*
//! scriptcraft-revisions 1
//! level=2 tracking=on
//! marks=12:1,15:2
//! */
//! ```
//!
//! `marks` lists element index and revision level. A file edited elsewhere may shift indices;
//! marks past the end are dropped.

use scriptcraft_model::{Element, ElementKind, Revisions, Screenplay};

const HEADER: &str = "scriptcraft-revisions 1";

/// Take the revision block off the end of a just-read script and apply it.
pub fn take(sp: &mut Screenplay) {
    let Some(last) = sp.elements.last() else { return };
    if last.kind != ElementKind::Boneyard || !last.text.trim_start().starts_with(HEADER) {
        return;
    }
    let text = last.text.clone();
    sp.elements.pop();
    for line in text.lines().skip(1) {
        for field in line.split_whitespace() {
            let Some((key, value)) = field.split_once('=') else { continue };
            match key {
                "level" => sp.revisions.level = value.parse().unwrap_or(0),
                "tracking" => sp.revisions.tracking = value == "on",
                "marks" => {
                    for mark in value.split(',') {
                        let Some((i, level)) = mark.split_once(':') else { continue };
                        let (Ok(i), Ok(level)) = (i.parse::<usize>(), level.parse::<u8>()) else { continue };
                        if let Some(e) = sp.elements.get_mut(i) {
                            e.revision = level;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// A copy of the script with the revision block appended, when there is anything to keep.
pub fn with_block(sp: &Screenplay) -> Option<Screenplay> {
    let marks: Vec<String> = sp.elements.iter().enumerate().filter(|(_, e)| e.revision > 0).map(|(i, e)| format!("{i}:{}", e.revision)).collect();
    if marks.is_empty() && sp.revisions == Revisions::default() {
        return None;
    }
    let mut out = sp.clone();
    let text =
        format!("{HEADER}\nlevel={} tracking={}\nmarks={}", sp.revisions.level, if sp.revisions.tracking { "on" } else { "off" }, marks.join(","));
    out.elements.push(Element::new(ElementKind::Boneyard, text));
    Some(out)
}

/// Mark the elements of `after` that are not in `before` (by kind and text) with the current
/// revision. Used after structural edits while tracking.
pub fn mark_changes(before: &Screenplay, after: &mut Screenplay) {
    if !after.revisions.tracking || after.revisions.level == 0 {
        return;
    }
    let level = after.revisions.level;
    let old: std::collections::HashSet<(ElementKind, &str)> = before.elements.iter().map(|e| (e.kind, e.text.as_str())).collect();
    for e in &mut after.elements {
        if !old.contains(&(e.kind, e.text.as_str())) && e.kind != ElementKind::PageBreak {
            e.revision = level;
        }
    }
}

/// Start the next revision (Blue, then Pink…) and turn tracking on.
pub fn start_next(sp: &mut Screenplay) -> u8 {
    sp.revisions.level = sp.revisions.level.saturating_add(1);
    sp.revisions.tracking = true;
    sp.revisions.level
}

/// Clear every mark (the script becomes the new clean draft); the level stays.
pub fn clear_marks(sp: &mut Screenplay) -> usize {
    let mut n = 0;
    for e in &mut sp.elements {
        if e.revision > 0 {
            e.revision = 0;
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_survive_fountain() {
        let mut sp = scriptcraft_fountain::parse("INT. A - DAY\n\nOne.\n\nTwo.\n");
        start_next(&mut sp);
        sp.elements[2].revision = 1;
        let text = scriptcraft_fountain::write(&with_block(&sp).unwrap());
        assert!(text.contains("/*\nscriptcraft-revisions 1"));
        let mut back = scriptcraft_fountain::parse(&text);
        take(&mut back);
        assert_eq!(back, sp);
    }

    #[test]
    fn nothing_to_keep_writes_nothing_and_bad_blocks_are_harmless() {
        let sp = scriptcraft_fountain::parse("INT. A - DAY\n");
        assert!(with_block(&sp).is_none());
        let mut odd = scriptcraft_fountain::parse("One.\n\n/*\nscriptcraft-revisions 1\nlevel=x marks=99:3,bad,1:z\n*/\n");
        take(&mut odd);
        assert_eq!(odd.elements.len(), 1);
        assert_eq!(odd.revisions.level, 0);
    }

    #[test]
    fn changed_elements_are_marked_while_tracking() {
        let before = scriptcraft_fountain::parse("INT. A - DAY\n\nOne.\n");
        let mut after = before.clone();
        after.revisions = Revisions { level: 2, tracking: true };
        after.elements[1].text = "One, changed.".into();
        mark_changes(&before, &mut after);
        assert_eq!((after.elements[0].revision, after.elements[1].revision), (0, 2));
    }
}
