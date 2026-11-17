//! [`Screenplay`] → Fountain text.

use scriptcraft_model::{Element, ElementKind, Screenplay, TitlePage};

use crate::classify::{Natural, classify, is_natural_cue};
use crate::read::{is_known_key, title_key};

/// Write a screenplay as Fountain. Paragraphs are separated by one blank line and the text ends
/// with a newline.
pub fn write(sp: &Screenplay) -> String {
    let mut paragraphs: Vec<String> = Vec::new();
    if !sp.title_page.is_empty() {
        paragraphs.push(write_title_page(&sp.title_page));
    }
    let elements = &sp.elements;
    let mut i = 0;
    while let Some(e) = elements.get(i) {
        if e.kind == ElementKind::Character {
            let speech_end = elements
                .iter()
                .skip(i + 1)
                .position(|n| !matches!(n.kind, ElementKind::Parenthetical | ElementKind::Dialogue | ElementKind::Lyric))
                .map_or(elements.len(), |p| i + 1 + p);
            paragraphs.push(write_speech(e, elements.get(i + 1..speech_end).unwrap_or_default()));
            i = speech_end;
            continue;
        }
        let mut paragraph = write_element(e);
        // Without a title page, a first paragraph that looks like one must not read as one.
        if paragraphs.is_empty() && e.kind == ElementKind::Action && title_key(&paragraph).is_some_and(|(k, _)| is_known_key(k)) {
            paragraph.insert(0, '!');
        }
        paragraphs.push(paragraph);
        i += 1;
    }
    let mut out = paragraphs.join("\n\n");
    out.push('\n');
    out
}

fn write_title_page(tp: &TitlePage) -> String {
    let mut lines: Vec<String> = Vec::new();
    for field in &tp.fields {
        if field.value.contains('\n') {
            lines.push(format!("{}:", field.key));
            lines.extend(field.value.split('\n').map(|l| format!("    {l}")));
        } else {
            lines.push(format!("{}: {}", field.key, field.value));
        }
    }
    lines.join("\n")
}

/// Lines of a paragraph; empty lines become two spaces so the paragraph stays one paragraph.
fn paragraph_lines(text: &str) -> String {
    text.split('\n').map(|l| if l.trim().is_empty() { "  " } else { l }).collect::<Vec<_>>().join("\n")
}

fn natural(text: &str) -> Natural {
    let lines = text.split('\n').count();
    let first = text.split('\n').next().unwrap_or("").trim();
    classify(first, lines, text)
}

fn write_element(e: &Element) -> String {
    match e.kind {
        ElementKind::SceneHeading => {
            let mut line = e.text.clone();
            if let Some(n) = &e.scene_number {
                line = format!("{line} #{n}#");
            }
            if line == line.trim() && natural(&line) == Natural::Heading { line } else { format!(".{line}") }
        }
        ElementKind::Action | ElementKind::Dialogue | ElementKind::Parenthetical => write_action(&e.text),
        ElementKind::Character => write_speech(e, &[]),
        ElementKind::Transition => {
            if natural(&e.text) == Natural::Transition && e.text == e.text.trim() {
                e.text.clone()
            } else {
                format!(">{}", e.text)
            }
        }
        ElementKind::Shot => {
            // Shots have no forcing mark: an all-capitals line that starts with a shot word.
            if natural(&e.text) == Natural::Shot && e.text == e.text.trim() { e.text.clone() } else { e.text.trim().to_uppercase() }
        }
        ElementKind::Centered => e.text.split('\n').map(|l| format!("> {l} <")).collect::<Vec<_>>().join("\n"),
        ElementKind::Lyric => lyric_lines(&e.text),
        ElementKind::Section => format!("{} {}", "#".repeat(usize::from(e.level.clamp(1, 6))), e.text),
        ElementKind::Synopsis => format!("= {}", e.text),
        ElementKind::Note => format!("[[{}]]", e.text),
        ElementKind::Boneyard => format!("/*\n{}\n*/", e.text),
        ElementKind::PageBreak => "===".to_owned(),
    }
}

fn lyric_lines(text: &str) -> String {
    text.split('\n').map(|l| format!("~{l}")).collect::<Vec<_>>().join("\n")
}

fn write_action(text: &str) -> String {
    let body = paragraph_lines(text);
    let first_blank = text.split('\n').next().is_none_or(|l| l.trim().is_empty());
    if !first_blank && natural(&body) == Natural::Action { body } else { format!("!{body}") }
}

fn write_speech(character: &Element, speech: &[Element]) -> String {
    let mut cue = character.text.clone();
    if character.dual {
        cue.push_str(" ^");
    }
    let mut lines: Vec<String> = Vec::new();
    for e in speech {
        match e.kind {
            ElementKind::Parenthetical => {
                let t = e.text.trim();
                lines.push(if t.starts_with('(') && t.ends_with(')') { t.to_owned() } else { format!("({t})") });
            }
            ElementKind::Lyric => lines.push(lyric_lines(&e.text)),
            _ => lines.push(paragraph_lines(&e.text)),
        }
    }
    let natural_cue = !lines.is_empty() && cue == cue.trim() && is_natural_cue(&cue) && {
        let mut probe = vec![cue.clone()];
        probe.extend(lines.iter().cloned());
        natural(&probe.join("\n")) == Natural::Character
    };
    if !natural_cue {
        cue.insert(0, '@');
    }
    let mut out = vec![cue];
    out.extend(lines);
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use scriptcraft_model::TitleField;

    fn sp(elements: Vec<Element>) -> Screenplay {
        Screenplay { title_page: TitlePage::default(), elements }
    }

    #[track_caller]
    fn assert_round_trip(s: &Screenplay) {
        let text = write(s);
        assert_eq!(&parse(&text), s, "Fountain written:\n{text}");
    }

    #[test]
    fn natural_forms() {
        let s = sp(vec![
            Element::new(ElementKind::SceneHeading, "INT. LIGHTHOUSE - NIGHT"),
            Element::new(ElementKind::Action, "Wind."),
            Element::new(ElementKind::Character, "MAYA"),
            Element::new(ElementKind::Dialogue, "Hello?"),
            Element::new(ElementKind::Transition, "CUT TO:"),
        ]);
        assert_eq!(write(&s), "INT. LIGHTHOUSE - NIGHT\n\nWind.\n\nMAYA\nHello?\n\nCUT TO:\n");
        assert_round_trip(&s);
    }

    #[test]
    fn forced_forms_round_trip() {
        let s = sp(vec![
            Element::new(ElementKind::SceneHeading, "The dream"),
            Element::new(ElementKind::Action, "INT. this looks like a heading"),
            Element::new(ElementKind::Action, "BANG!\nThe door gives way."),
            Element::new(ElementKind::Character, "McCLANE"),
            Element::new(ElementKind::Dialogue, "Yes."),
            Element::new(ElementKind::Character, "NARRATOR"),
            Element::new(ElementKind::Transition, "Fade out."),
            Element::new(ElementKind::Action, "CUT TO:"),
            Element::new(ElementKind::Action, "> centered? <"),
            Element::new(ElementKind::Action, "!bang"),
            Element::new(ElementKind::Action, ""),
            Element::new(ElementKind::Action, "CLOSE ON A DOOR"),
            Element::new(ElementKind::Action, "[[only a note]]"),
        ]);
        assert_round_trip(&s);
    }

    #[test]
    fn everything_else_round_trips() {
        let mut dual = Element::new(ElementKind::Character, "TOMAS");
        dual.dual = true;
        let mut s = sp(vec![
            Element::section("Act One", 1),
            Element::section("The storm", 2),
            Element::new(ElementKind::Synopsis, "Maya finds the lantern."),
            Element::numbered_heading("EXT. CLIFF - DAWN", "4A"),
            Element::new(ElementKind::Shot, "CLOSE ON THE LANTERN"),
            Element::new(ElementKind::Action, "It flickers.\n\nThen holds."),
            Element::new(ElementKind::Character, "MAYA (V.O.)"),
            Element::new(ElementKind::Parenthetical, "(whispering)"),
            Element::new(ElementKind::Dialogue, "Stay lit.\n\nPlease."),
            Element::new(ElementKind::Lyric, "Little light, little light"),
            dual,
            Element::new(ElementKind::Dialogue, "It will."),
            Element::new(ElementKind::Note, "check the tide tables"),
            Element::new(ElementKind::Boneyard, "An older ending.\n\nWith two paragraphs."),
            Element::new(ElementKind::PageBreak, ""),
            Element::new(ElementKind::Lyric, "Sing it once\nsing it twice"),
            Element::new(ElementKind::Centered, "THE END"),
        ]);
        s.title_page.fields = vec![
            TitleField { key: "Title".into(), value: "The Lantern".into() },
            TitleField { key: "Contact".into(), value: "12 Example Road\nSpringfield".into() },
        ];
        assert_round_trip(&s);
    }

    #[test]
    fn untitled_script_starting_with_a_key_like_line() {
        let s = sp(vec![Element::new(ElementKind::Action, "Title: a card on the screen"), Element::new(ElementKind::Action, "x")]);
        assert_round_trip(&s);
    }
}
