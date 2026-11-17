#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! The screenplay document model.
//!
//! A [`Screenplay`] is a [`TitlePage`] plus a flat list of [`Element`]s in reading order. Every
//! element has one [`ElementKind`] and its text. Text is stored as typed (a character name may be
//! lowercase in the model); upper-casing is a formatting rule applied by layout and the editor.
//!
//! This crate is the bottom layer: no I/O, no UI, no file formats.

use serde::{Deserialize, Serialize};

/// What an element is. The order is the order the element menu shows them in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementKind {
    /// `INT. KITCHEN - NIGHT`.
    SceneHeading,
    /// Description of what is seen and heard.
    Action,
    /// The speaker's name above dialogue, with an optional extension: `MAYA (V.O.)`.
    Character,
    /// A short direction inside dialogue: `(quietly)`. Stored with its parentheses.
    Parenthetical,
    /// What a character says.
    Dialogue,
    /// `CUT TO:`.
    Transition,
    /// A camera direction inside a scene: `CLOSE ON THE KEY`.
    Shot,
    /// Centered text (`THE END`).
    Centered,
    /// Sung lines.
    Lyric,
    /// An outline section heading; not printed. Level 1 is the outermost.
    Section,
    /// An outline synopsis; not printed.
    Synopsis,
    /// A writer's note on its own; not printed.
    Note,
    /// Text kept in the file but left out of the script; not printed.
    Boneyard,
    /// A forced page break.
    PageBreak,
}

impl ElementKind {
    /// Every kind, in menu order.
    pub const ALL: [ElementKind; 14] = [
        ElementKind::SceneHeading,
        ElementKind::Action,
        ElementKind::Character,
        ElementKind::Parenthetical,
        ElementKind::Dialogue,
        ElementKind::Transition,
        ElementKind::Shot,
        ElementKind::Centered,
        ElementKind::Lyric,
        ElementKind::Section,
        ElementKind::Synopsis,
        ElementKind::Note,
        ElementKind::Boneyard,
        ElementKind::PageBreak,
    ];

    /// The kinds a writer picks from while typing (the element menu and Tab cycling).
    pub const WRITING: [ElementKind; 7] = [
        ElementKind::SceneHeading,
        ElementKind::Action,
        ElementKind::Character,
        ElementKind::Parenthetical,
        ElementKind::Dialogue,
        ElementKind::Transition,
        ElementKind::Shot,
    ];

    /// The label shown to users.
    pub fn label(self) -> &'static str {
        match self {
            ElementKind::SceneHeading => "Scene Heading",
            ElementKind::Action => "Action",
            ElementKind::Character => "Character",
            ElementKind::Parenthetical => "Parenthetical",
            ElementKind::Dialogue => "Dialogue",
            ElementKind::Transition => "Transition",
            ElementKind::Shot => "Shot",
            ElementKind::Centered => "Centered",
            ElementKind::Lyric => "Lyric",
            ElementKind::Section => "Section",
            ElementKind::Synopsis => "Synopsis",
            ElementKind::Note => "Note",
            ElementKind::Boneyard => "Omitted Text",
            ElementKind::PageBreak => "Page Break",
        }
    }

    /// A stable id for commands, settings and interchange files (`scene_heading`).
    pub fn id(self) -> &'static str {
        match self {
            ElementKind::SceneHeading => "scene_heading",
            ElementKind::Action => "action",
            ElementKind::Character => "character",
            ElementKind::Parenthetical => "parenthetical",
            ElementKind::Dialogue => "dialogue",
            ElementKind::Transition => "transition",
            ElementKind::Shot => "shot",
            ElementKind::Centered => "centered",
            ElementKind::Lyric => "lyric",
            ElementKind::Section => "section",
            ElementKind::Synopsis => "synopsis",
            ElementKind::Note => "note",
            ElementKind::Boneyard => "boneyard",
            ElementKind::PageBreak => "page_break",
        }
    }

    /// The kind with this [`ElementKind::id`].
    pub fn from_id(id: &str) -> Option<ElementKind> {
        ElementKind::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Whether the text is shown and printed in capitals.
    pub fn is_uppercase(self) -> bool {
        matches!(self, ElementKind::SceneHeading | ElementKind::Character | ElementKind::Transition | ElementKind::Shot)
    }

    /// Whether the element appears in the printed script.
    pub fn is_printed(self) -> bool {
        !matches!(self, ElementKind::Section | ElementKind::Synopsis | ElementKind::Note | ElementKind::Boneyard | ElementKind::PageBreak)
    }

    /// Whether the element belongs to a speech (character, parenthetical, dialogue, lyric).
    pub fn is_speech(self) -> bool {
        matches!(self, ElementKind::Character | ElementKind::Parenthetical | ElementKind::Dialogue | ElementKind::Lyric)
    }
}

/// One paragraph of the script.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    pub kind: ElementKind,
    /// The text, with `\n` between lines the writer broke by hand.
    pub text: String,
    /// A locked scene number (`12A`) on a scene heading; `None` numbers scenes in order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_number: Option<String>,
    /// On a character: this speech sits beside the previous one (dual dialogue).
    #[serde(default, skip_serializing_if = "is_false")]
    pub dual: bool,
    /// On a section: its depth, 1 for the outermost.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub level: u8,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

impl Element {
    /// A plain element of this kind.
    pub fn new(kind: ElementKind, text: impl Into<String>) -> Element {
        let level = if kind == ElementKind::Section { 1 } else { 0 };
        Element { kind, text: text.into(), scene_number: None, dual: false, level }
    }

    /// A scene heading with a locked scene number.
    pub fn numbered_heading(text: impl Into<String>, number: impl Into<String>) -> Element {
        Element { scene_number: Some(number.into()), ..Element::new(ElementKind::SceneHeading, text) }
    }

    /// A section at this depth (clamped to 1..=6).
    pub fn section(text: impl Into<String>, level: u8) -> Element {
        Element { level: level.clamp(1, 6), ..Element::new(ElementKind::Section, text) }
    }

    /// Change the kind, keeping the text and fixing the fields that only one kind uses.
    pub fn set_kind(&mut self, kind: ElementKind) {
        self.kind = kind;
        if kind != ElementKind::SceneHeading {
            self.scene_number = None;
        }
        if kind != ElementKind::Character {
            self.dual = false;
        }
        self.level = if kind == ElementKind::Section { self.level.clamp(1, 6) } else { 0 };
    }

    /// The text as it is displayed and printed: in capitals for headings, characters, transitions
    /// and shots.
    pub fn display_text(&self) -> String {
        if self.kind.is_uppercase() { self.text.to_uppercase() } else { self.text.clone() }
    }

    /// For a character element, the name without its extension: `MAYA` for `Maya (V.O.)`.
    pub fn character_name(&self) -> Option<String> {
        if self.kind != ElementKind::Character {
            return None;
        }
        let name = character_name(&self.text);
        if name.is_empty() { None } else { Some(name) }
    }
}

/// The name part of a character cue, in capitals and without extensions or a dual-dialogue mark:
/// `MAYA` for `Maya (V.O.)`, `DR. OKAFOR` for `DR. OKAFOR (CONT'D) ^`.
pub fn character_name(cue: &str) -> String {
    let base = cue.split('(').next().unwrap_or(cue);
    base.trim().trim_end_matches('^').trim().to_uppercase()
}

/// One key of the title page: `Title`, `Credit`, `Author`, `Source`, `Draft date`, `Contact`, or
/// any other key the file used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleField {
    pub key: String,
    /// The value, with `\n` between its lines.
    pub value: String,
}

/// The title page: ordered keys as written.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitlePage {
    pub fields: Vec<TitleField>,
}

impl TitlePage {
    /// The keys the title page dialog offers, in order.
    pub const STANDARD_KEYS: [&'static str; 7] = ["Title", "Credit", "Author", "Source", "Draft date", "Contact", "Notes"];

    /// The value of the first field with this key, compared without case.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.iter().find(|f| f.key.eq_ignore_ascii_case(key)).map(|f| f.value.as_str())
    }

    /// Set a key's value, adding the key at the end when it is new. An empty value removes it.
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        if value.is_empty() {
            self.fields.retain(|f| !f.key.eq_ignore_ascii_case(key));
            return;
        }
        match self.fields.iter_mut().find(|f| f.key.eq_ignore_ascii_case(key)) {
            Some(field) => field.value = value,
            None => self.fields.push(TitleField { key: key.to_owned(), value }),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A whole script.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screenplay {
    pub title_page: TitlePage,
    pub elements: Vec<Element>,
}

impl Screenplay {
    /// The title from the title page, if any.
    pub fn title(&self) -> Option<&str> {
        self.title_page.get("Title").map(str::trim).filter(|t| !t.is_empty())
    }

    /// Indices of the scene headings, in order.
    pub fn scene_heading_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.elements.iter().enumerate().filter(|(_, e)| e.kind == ElementKind::SceneHeading).map(|(i, _)| i)
    }

    /// The index of the scene heading that element `index` belongs to, if it is inside a scene.
    pub fn scene_of(&self, index: usize) -> Option<usize> {
        let end = index.saturating_add(1).min(self.elements.len());
        self.elements.get(..end)?.iter().rposition(|e| e.kind == ElementKind::SceneHeading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_name_drops_extensions_and_dual_mark() {
        assert_eq!(character_name("Maya (V.O.)"), "MAYA");
        assert_eq!(character_name("DR. OKAFOR (CONT'D) ^"), "DR. OKAFOR");
        assert_eq!(character_name("  ^ "), "");
    }

    #[test]
    fn title_page_set_get_and_remove() {
        let mut tp = TitlePage::default();
        tp.set("Title", "The Lantern");
        tp.set("title", "The Lantern Keeper");
        assert_eq!(tp.get("TITLE"), Some("The Lantern Keeper"));
        assert_eq!(tp.fields.len(), 1);
        tp.set("Title", "");
        assert!(tp.is_empty());
    }

    #[test]
    fn scene_of_finds_enclosing_heading() {
        let sp = Screenplay {
            title_page: TitlePage::default(),
            elements: vec![
                Element::new(ElementKind::Action, "Before any scene."),
                Element::new(ElementKind::SceneHeading, "INT. A - DAY"),
                Element::new(ElementKind::Action, "x"),
            ],
        };
        assert_eq!(sp.scene_of(0), None);
        assert_eq!(sp.scene_of(2), Some(1));
        assert_eq!(sp.scene_of(99), Some(1));
    }

    #[test]
    fn set_kind_clears_fields_of_other_kinds() {
        let mut e = Element::numbered_heading("INT. A - DAY", "4");
        e.set_kind(ElementKind::Action);
        assert_eq!(e.scene_number, None);
        e.set_kind(ElementKind::Section);
        assert_eq!(e.level, 1);
    }

    #[test]
    fn kind_ids_round_trip() {
        for kind in ElementKind::ALL {
            assert_eq!(ElementKind::from_id(kind.id()), Some(kind));
        }
    }
}
