#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! Import of FDX files, the XML screenplay format of a widely used commercial screenwriting
//! application, into [`scriptcraft_model::Screenplay`].
//!
//! Written from the publicly observable structure of the format (a `FinalDraft` root, a
//! `Content` list of `Paragraph Type="…"` elements with `Text` runs, `DualDialogue` groups and a
//! `TitlePage`), not from any implementation. XML is read with craft-libs' `craft-xml`, which
//! bounds size, depth and node count and expands no entities.
//!
//! Text styling (bold, italic, underline runs), revision marks, script notes and page layout
//! settings are not imported yet; see `docs/file-formats.md`.

use craft_xml::El;
use scriptcraft_model::{Element, ElementKind, Screenplay, TitlePage};

/// The largest FDX file read: 64 MiB.
pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// Why an FDX file was not read.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the file is {0} bytes; FDX files over {MAX_INPUT_BYTES} bytes are not opened")]
    TooLarge(usize),
    #[error("the file is not valid XML: {0}")]
    Xml(#[from] craft_xml::Error),
    #[error("the XML is not an FDX screenplay (its root element is <{0}>)")]
    NotFdx(String),
}

/// What an FDX import kept and dropped, for the import message.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Paragraph types read as action because ScriptCraft has no such element.
    pub unknown_types: Vec<String>,
    /// Empty paragraphs skipped.
    pub empty_paragraphs: usize,
}

/// Read an FDX file.
pub fn import(bytes: &[u8]) -> Result<(Screenplay, ImportReport), Error> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Error::TooLarge(bytes.len()));
    }
    let root = craft_xml::parse(bytes)?;
    if root.local() != "FinalDraft" {
        return Err(Error::NotFdx(root.name.clone()));
    }
    let mut report = ImportReport::default();
    let mut elements = Vec::new();
    if let Some(content) = root.child("Content") {
        read_paragraphs(content, &mut elements, &mut report, false);
    }
    let title_page = root.path(&["TitlePage", "Content"]).map(read_title_page).unwrap_or_default();
    Ok((Screenplay { title_page, elements }, report))
}

fn kind_of(fdx_type: &str) -> Option<ElementKind> {
    Some(match fdx_type {
        "Scene Heading" => ElementKind::SceneHeading,
        "Action" | "General" | "Cast List" => ElementKind::Action,
        "Character" => ElementKind::Character,
        "Parenthetical" => ElementKind::Parenthetical,
        "Dialogue" => ElementKind::Dialogue,
        "Transition" => ElementKind::Transition,
        "Shot" => ElementKind::Shot,
        "Lyrics" | "Lyric" => ElementKind::Lyric,
        "New Act" | "End of Act" => ElementKind::Centered,
        _ => return None,
    })
}

/// The paragraph's own text: its `Text` runs joined, line breaks normalised. Nested
/// paragraphs (scene summaries) are not part of it.
fn paragraph_text(p: &El) -> String {
    let mut s = String::new();
    for run in p.children("Text") {
        s.push_str(&run.text());
    }
    s.replace("\r\n", "\n").replace('\r', "\n").trim().to_owned()
}

/// Paragraphs of a `Content` (or `DualDialogue`). Nesting is one level deep by format; deeper
/// `DualDialogue` groups are read flat rather than followed, so depth stays bounded.
fn read_paragraphs(content: &El, out: &mut Vec<Element>, report: &mut ImportReport, in_dual: bool) {
    let mut characters_in_group = 0usize;
    for p in content.children("Paragraph") {
        if let Some(dual) = p.child("DualDialogue")
            && !in_dual
        {
            read_paragraphs(dual, out, report, true);
            continue;
        }
        let fdx_type = p.attr("Type").unwrap_or("Action");
        let kind = kind_of(fdx_type).unwrap_or_else(|| {
            if !report.unknown_types.iter().any(|t| t == fdx_type) {
                report.unknown_types.push(fdx_type.to_owned());
            }
            ElementKind::Action
        });
        let text = paragraph_text(p);
        if text.is_empty() && kind != ElementKind::Dialogue {
            report.empty_paragraphs += 1;
            continue;
        }
        let mut e = Element::new(kind, text);
        if kind == ElementKind::SceneHeading {
            e.scene_number = p.attr("Number").map(str::trim).filter(|n| !n.is_empty()).map(str::to_owned);
        }
        if kind == ElementKind::Character && in_dual {
            e.dual = characters_in_group > 0;
            characters_in_group += 1;
        }
        out.push(e);
    }
}

/// FDX title pages are free-form paragraphs. Centered ones are read in order as the title, the
/// credit ("Written by", "by"), the author and the source; left-aligned ones as contact details and
/// right-aligned ones as the draft date.
fn read_title_page(content: &El) -> TitlePage {
    let mut centered: Vec<String> = Vec::new();
    let mut left: Vec<String> = Vec::new();
    let mut right: Vec<String> = Vec::new();
    for p in content.children("Paragraph") {
        let text = paragraph_text(p);
        if text.is_empty() {
            continue;
        }
        match p.attr("Alignment").unwrap_or("Left") {
            "Center" => centered.push(text),
            "Right" => right.push(text),
            _ => left.push(text),
        }
    }
    let mut tp = TitlePage::default();
    let mut rest = centered.into_iter();
    if let Some(title) = rest.next() {
        tp.set("Title", title);
    }
    let mut remaining: Vec<String> = rest.collect();
    let credit_at = remaining.iter().position(|t| {
        let t = t.to_lowercase();
        t == "by" || t.starts_with("written by") || t.starts_with("screenplay by") || t.starts_with("story by")
    });
    if let Some(i) = credit_at {
        tp.set("Credit", remaining.remove(i));
        if i < remaining.len() {
            tp.set("Author", remaining.remove(i));
        }
    } else if !remaining.is_empty() {
        tp.set("Author", remaining.remove(0));
    }
    tp.set("Source", remaining.join("\n"));
    tp.set("Contact", left.join("\n"));
    tp.set("Draft date", right.join("\n"));
    tp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_other_xml() {
        assert!(matches!(import(b"<html/>"), Err(Error::NotFdx(name)) if name == "html"));
        assert!(matches!(import(b""), Err(Error::Xml(_))));
        // craft-xml recovers what it can from an unclosed document; that must not panic.
        assert!(import(b"<FinalDraft><Content>").is_ok_and(|(sp, _)| sp.elements.is_empty()));
    }

    #[test]
    fn unknown_types_become_action_and_are_reported() {
        let (sp, report) = import(br#"<FinalDraft><Content><Paragraph Type="Montage Header"><Text>MONTAGE</Text></Paragraph><Paragraph Type="Action"><Text></Text></Paragraph></Content></FinalDraft>"#).unwrap();
        assert_eq!(sp.elements, vec![Element::new(ElementKind::Action, "MONTAGE")]);
        assert_eq!(report.unknown_types, ["Montage Header"]);
        assert_eq!(report.empty_paragraphs, 1);
    }
}
