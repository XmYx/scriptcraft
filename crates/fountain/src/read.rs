//! Fountain text → [`Screenplay`].

use scriptcraft_model::{Element, ElementKind, Screenplay, TitleField, TitlePage};

use crate::classify::{Natural, classify};
use crate::{Error, MAX_INPUT_BYTES};

/// Read a Fountain file's bytes: at most [`MAX_INPUT_BYTES`], UTF-8 with or without a byte-order
/// mark (invalid sequences become U+FFFD instead of failing).
pub fn parse_bytes(bytes: &[u8]) -> Result<Screenplay, Error> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Error::TooLarge(bytes.len()));
    }
    Ok(parse(&String::from_utf8_lossy(bytes)))
}

/// Read Fountain text. Never fails: text that is nothing else is action.
pub fn parse(text: &str) -> Screenplay {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let (title_page, start) = read_title_page(&lines);
    let elements = read_body(lines.get(start..).unwrap_or_default());
    Screenplay { title_page, elements }
}

/// Title page keys that mark the first paragraph as a title page. Other keys are kept once a
/// known one is present.
const KNOWN_KEYS: [&str; 16] = [
    "title",
    "credit",
    "author",
    "authors",
    "source",
    "draft date",
    "date",
    "contact",
    "contact info",
    "copyright",
    "notes",
    "revision",
    "format",
    "font",
    "episode",
    "series",
];

/// `Key: value` at the start of a line. Keys are short, start with a letter and hold letters,
/// digits, spaces, `-` and `_`.
pub(crate) fn title_key(line: &str) -> Option<(&str, &str)> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (key, value) = line.split_once(':')?;
    let key = key.trim();
    let first = key.chars().next()?;
    let valid = first.is_alphabetic() && key.len() <= 40 && key.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'));
    valid.then_some((key, value.trim()))
}

pub(crate) fn is_known_key(key: &str) -> bool {
    let key = key.to_lowercase();
    KNOWN_KEYS.contains(&key.as_str())
}

fn is_continuation(line: &str) -> bool {
    (line.starts_with("   ") || line.starts_with('\t')) && !line.trim().is_empty()
}

fn read_title_page(lines: &[&str]) -> (TitlePage, usize) {
    let end = lines.iter().position(|l| l.trim().is_empty()).unwrap_or(lines.len());
    let Some(block) = lines.get(..end) else { return (TitlePage::default(), 0) };
    let starts_with_key = block.first().is_some_and(|l| title_key(l).is_some());
    let well_formed = block.iter().all(|l| title_key(l).is_some() || is_continuation(l));
    let has_known = block.iter().filter_map(|l| title_key(l)).any(|(k, _)| is_known_key(k));
    if !(starts_with_key && well_formed && has_known) {
        return (TitlePage::default(), 0);
    }
    let mut fields: Vec<TitleField> = Vec::new();
    for line in block {
        if let Some((key, value)) = title_key(line) {
            fields.push(TitleField { key: key.to_owned(), value: value.to_owned() });
        } else if let Some(field) = fields.last_mut() {
            if !field.value.is_empty() {
                field.value.push('\n');
            }
            field.value.push_str(line.trim());
        }
    }
    (TitlePage { fields }, end)
}

/// A whitespace-only line of two or more characters inside a paragraph keeps the paragraph going
/// (Fountain's "two spaces" rule), when more text of the paragraph follows.
fn is_inner_blank(lines: &[&str], i: usize) -> bool {
    let Some(line) = lines.get(i) else { return false };
    if !(line.trim().is_empty() && line.len() >= 2) {
        return false;
    }
    let mut j = i + 1;
    while let Some(next) = lines.get(j) {
        if !next.trim().is_empty() {
            return true;
        }
        if next.len() < 2 {
            return false;
        }
        j += 1;
    }
    false
}

fn read_body(lines: &[&str]) -> Vec<Element> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(line) = lines.get(i) {
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        if line.trim_start().starts_with("/*") {
            i = read_boneyard(lines, i, &mut out);
            continue;
        }
        let mut block: Vec<&str> = Vec::new();
        while let Some(line) = lines.get(i) {
            if !line.trim().is_empty() {
                block.push(line.trim_end());
            } else if !block.is_empty() && is_inner_blank(lines, i) {
                block.push("");
            } else {
                break;
            }
            i += 1;
        }
        read_block(&block, &mut out);
    }
    out
}

/// A `/* … */` span, which may cross blank lines. Unterminated, it runs to the end of the file.
fn read_boneyard(lines: &[&str], start: usize, out: &mut Vec<Element>) -> usize {
    let first = lines.get(start).map_or("", |l| l.trim_start());
    let after = first.get(2..).unwrap_or("");
    if let Some(end) = after.find("*/") {
        out.push(Element::new(ElementKind::Boneyard, after.get(..end).unwrap_or("").trim()));
        return start + 1;
    }
    let mut parts: Vec<&str> = Vec::new();
    if !after.trim().is_empty() {
        parts.push(after);
    }
    let mut j = start + 1;
    while let Some(line) = lines.get(j) {
        if let Some(end) = line.find("*/") {
            let before = line.get(..end).unwrap_or("");
            if !before.trim().is_empty() {
                parts.push(before);
            }
            out.push(Element::new(ElementKind::Boneyard, parts.join("\n")));
            return j + 1;
        }
        parts.push(line);
        j += 1;
    }
    out.push(Element::new(ElementKind::Boneyard, parts.join("\n")));
    lines.len()
}

/// Read one paragraph. A paragraph can hold several elements (a forced heading followed by more
/// lines); it is consumed in a loop, never by recursion, so a hostile file can't exhaust the stack.
fn read_block(block: &[&str], out: &mut Vec<Element>) {
    let mut rest = block;
    while !rest.is_empty() {
        let used = read_one(rest, out).max(1);
        rest = rest.get(used..).unwrap_or_default();
    }
}

/// Read the element(s) at the start of `block`; return how many lines were used.
fn read_one(block: &[&str], out: &mut Vec<Element>) -> usize {
    let Some(first_raw) = block.first() else { return 1 };
    let first = first_raw.trim();
    // Only a paragraph starting with `[[` needs its whole text (to tell a note from action); joining
    // every time would make a long paragraph of one-line elements quadratic.
    let whole = if first.starts_with("[[") { block.join("\n") } else { String::new() };
    match classify(first, block.len(), &whole) {
        Natural::PageBreak => {
            out.push(Element::new(ElementKind::PageBreak, ""));
            1
        }
        Natural::Section => {
            let level = first.chars().take_while(|c| *c == '#').count();
            let level = u8::try_from(level).unwrap_or(u8::MAX);
            out.push(Element::section(first.trim_start_matches('#').trim(), level));
            1
        }
        Natural::Synopsis => {
            out.push(Element::new(ElementKind::Synopsis, first.get(1..).unwrap_or("").trim()));
            1
        }
        Natural::Lyric => {
            let lyric: Vec<&str> = block.iter().take_while(|l| l.trim_start().starts_with('~')).map(|l| strip_first(l.trim_start())).collect();
            out.push(Element::new(ElementKind::Lyric, lyric.join("\n")));
            lyric.len()
        }
        Natural::Note => {
            let t = whole.trim();
            out.push(Element::new(ElementKind::Note, t.get(2..t.len().saturating_sub(2)).unwrap_or("")));
            block.len()
        }
        Natural::ForcedAction => {
            let mut lines: Vec<&str> = vec![strip_first(first_raw.trim_start())];
            lines.extend(block.iter().skip(1));
            out.push(Element::new(ElementKind::Action, lines.join("\n")));
            block.len()
        }
        Natural::ForcedCharacter => {
            read_speech(strip_first(first).trim(), block.get(1..).unwrap_or_default(), out);
            block.len()
        }
        Natural::ForcedHeading => {
            out.push(heading(strip_first(first).trim()));
            1
        }
        Natural::Centered => {
            let centered: Vec<&str> = block
                .iter()
                .map(|l| l.trim())
                .take_while(|l| l.starts_with('>') && l.ends_with('<'))
                .map(|l| l.get(1..l.len().saturating_sub(1)).unwrap_or("").trim())
                .collect();
            out.push(Element::new(ElementKind::Centered, centered.join("\n")));
            centered.len()
        }
        Natural::ForcedTransition => {
            out.push(Element::new(ElementKind::Transition, strip_first(first).trim()));
            1
        }
        Natural::Heading => {
            out.push(heading(first));
            1
        }
        Natural::Transition => {
            out.push(Element::new(ElementKind::Transition, first));
            1
        }
        Natural::Character => {
            read_speech(first, block.get(1..).unwrap_or_default(), out);
            block.len()
        }
        Natural::Shot => {
            out.push(Element::new(ElementKind::Shot, first));
            1
        }
        Natural::Boneyard | Natural::Action => {
            out.push(Element::new(ElementKind::Action, block.join("\n")));
            block.len()
        }
    }
}

/// `s` without its first character (a forcing mark).
fn strip_first(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next();
    chars.as_str()
}

/// A scene heading, with a trailing `#12A#` scene number split off.
fn heading(text: &str) -> Element {
    let (text, number) = split_scene_number(text);
    let mut e = Element::new(ElementKind::SceneHeading, text);
    e.scene_number = number.map(str::to_owned);
    e
}

pub(crate) fn split_scene_number(text: &str) -> (&str, Option<&str>) {
    let t = text.trim_end();
    if let Some(body) = t.strip_suffix('#')
        && let Some(pos) = body.rfind('#')
    {
        let number = body.get(pos + 1..).unwrap_or("");
        if !number.is_empty() && number.chars().all(is_scene_number_char) {
            return (body.get(..pos).unwrap_or("").trim_end(), Some(number));
        }
    }
    (t, None)
}

pub(crate) fn is_scene_number_char(c: char) -> bool {
    c.is_alphanumeric() || c == '.' || c == '-'
}

/// A character cue and the dialogue, parentheticals and lyrics under it.
fn read_speech(cue: &str, lines: &[&str], out: &mut Vec<Element>) {
    let (cue, dual) = match cue.strip_suffix('^') {
        Some(c) => (c.trim_end(), true),
        None => (cue, false),
    };
    let mut character = Element::new(ElementKind::Character, cue);
    character.dual = dual;
    out.push(character);
    let mut buffer: Vec<&str> = Vec::new();
    let mut buffer_kind = ElementKind::Dialogue;
    let flush = |buffer: &mut Vec<&str>, kind: ElementKind, out: &mut Vec<Element>| {
        if !buffer.is_empty() {
            out.push(Element::new(kind, buffer.join("\n")));
            buffer.clear();
        }
    };
    for line in lines {
        let t = line.trim();
        if t.len() >= 2 && t.starts_with('(') && t.ends_with(')') {
            flush(&mut buffer, buffer_kind, out);
            out.push(Element::new(ElementKind::Parenthetical, t));
            continue;
        }
        let (kind, text) = match t.strip_prefix('~') {
            Some(lyric) => (ElementKind::Lyric, lyric),
            None if t.is_empty() => (buffer_kind, t),
            None => (ElementKind::Dialogue, t),
        };
        if kind != buffer_kind {
            flush(&mut buffer, buffer_kind, out);
            buffer_kind = kind;
        }
        buffer.push(text);
    }
    flush(&mut buffer, buffer_kind, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(sp: &Screenplay) -> Vec<ElementKind> {
        sp.elements.iter().map(|e| e.kind).collect()
    }

    #[test]
    fn title_page_with_multiline_value() {
        let sp = parse("Title: The Lantern\nAuthor: Sam Rivera\nContact:\n    12 Example Road\n    Springfield\n\nINT. A - DAY\n");
        assert_eq!(sp.title(), Some("The Lantern"));
        assert_eq!(sp.title_page.get("contact"), Some("12 Example Road\nSpringfield"));
        assert_eq!(kinds(&sp), [ElementKind::SceneHeading]);
    }

    #[test]
    fn fade_in_alone_is_not_a_title_page() {
        let sp = parse("FADE IN:\n\nINT. A - DAY\n");
        assert!(sp.title_page.is_empty());
        assert_eq!(kinds(&sp), [ElementKind::Action, ElementKind::SceneHeading]);
    }

    #[test]
    fn speech_with_parenthetical_and_dual() {
        let sp = parse("MAYA\n(quietly)\nIt's late.\nGo home.\n\nTOMAS ^\nNo.\n");
        assert_eq!(
            kinds(&sp),
            [ElementKind::Character, ElementKind::Parenthetical, ElementKind::Dialogue, ElementKind::Character, ElementKind::Dialogue]
        );
        assert_eq!(sp.elements[2].text, "It's late.\nGo home.");
        assert!(sp.elements[3].dual);
        assert_eq!(sp.elements[3].text, "TOMAS");
    }

    #[test]
    fn scene_numbers() {
        let sp = parse("INT. HALL - NIGHT #12A#\n");
        assert_eq!(sp.elements[0].text, "INT. HALL - NIGHT");
        assert_eq!(sp.elements[0].scene_number.as_deref(), Some("12A"));
    }

    #[test]
    fn two_space_line_keeps_dialogue_together() {
        let sp = parse("MAYA\nOne.\n  \nTwo.\n");
        assert_eq!(sp.elements[1].text, "One.\n\nTwo.");
    }

    #[test]
    fn boneyard_spans_blank_lines() {
        let sp = parse("/*\nCut scene.\n\nStill cut.\n*/\n\nINT. A - DAY\n");
        assert_eq!(sp.elements[0].kind, ElementKind::Boneyard);
        assert_eq!(sp.elements[0].text, "Cut scene.\n\nStill cut.");
        assert_eq!(sp.elements[1].kind, ElementKind::SceneHeading);
    }

    #[test]
    fn crlf_and_bom() {
        let sp = parse("\u{feff}INT. A - DAY\r\n\r\nShe waits.\r\n");
        assert_eq!(kinds(&sp), [ElementKind::SceneHeading, ElementKind::Action]);
    }

    #[test]
    fn too_large_is_an_error() {
        let big = vec![b'a'; MAX_INPUT_BYTES + 1];
        assert_eq!(parse_bytes(&big), Err(Error::TooLarge(MAX_INPUT_BYTES + 1)));
    }

    #[test]
    fn deep_page_breaks_do_not_recurse() {
        let text = "===\n".repeat(200_000);
        let sp = parse(&text);
        assert_eq!(sp.elements.len(), 200_000);
    }
}
