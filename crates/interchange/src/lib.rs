#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! The scene list ScriptCraft hands to sibling apps, BoardCraft first: one entry per scene with
//! its number, heading (split into interior/exterior, location and time of day), pages, length in
//! eighths of a page, characters, action text and the scene's beats in order.
//!
//! The JSON shape is versioned (`"format": "scriptcraft-scenes", "version": 1`) and documented
//! in `docs/interop.md`. Fountain itself stays the primary interchange: the scene list is derived
//! from the same model and adds what a storyboard tool would otherwise recompute (page lengths,
//! stable scene ids).

use scriptcraft_layout::Layout;
use scriptcraft_model::{Element, ElementKind, Screenplay};
use serde::{Deserialize, Serialize};

/// `format` of the JSON document.
pub const FORMAT: &str = "scriptcraft-scenes";
/// `version` of the JSON document. Readers accept any document with the same major version and
/// ignore fields they don't know.
pub const VERSION: u32 = 1;
/// The note text prefix that carries a scene's stable id: `[[scene-id: 7f3a09c2b1d84e65]]`.
pub const SCENE_ID_PREFIX: &str = "scene-id:";

/// The whole scene list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneList {
    pub format: String,
    pub version: u32,
    /// `ScriptCraft <version>`.
    pub generator: String,
    #[serde(default)]
    pub title: Option<String>,
    /// File the scenes came from, when known.
    #[serde(default)]
    pub source: Option<String>,
    /// `us-letter` or `a4`: the paper the pages and eighths were counted on.
    pub paper: String,
    pub page_count: usize,
    pub scenes: Vec<Scene>,
}

/// One scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// 1-based position in the script.
    pub index: usize,
    /// The scene number: the heading's locked number, or its position.
    pub number: String,
    /// Stable id from a `[[scene-id: …]]` note right after the heading; `null` when the scene
    /// has none.
    #[serde(default)]
    pub id: Option<String>,
    /// The heading as printed (`INT. LIGHTHOUSE - LAMP ROOM - NIGHT`).
    pub heading: String,
    pub setting: Setting,
    pub page_start: u32,
    pub page_end: u32,
    /// Length in eighths of a page (at least 1).
    pub length_eighths: u32,
    /// The first synopsis inside the scene.
    #[serde(default)]
    pub synopsis: Option<String>,
    /// Speaking characters in order of first line, names without extensions.
    pub characters: Vec<String>,
    /// The scene's action paragraphs, separated by blank lines.
    pub action: String,
    /// Everything printed in the scene, in order, grouped the way a storyboard panel would be.
    pub beats: Vec<Beat>,
    /// Index of the heading element in the model; not part of the file.
    #[serde(skip)]
    pub element: usize,
}

/// A heading split into its parts. `INT. LIGHTHOUSE - LAMP ROOM - NIGHT` is `INT.`,
/// `LIGHTHOUSE - LAMP ROOM` and `NIGHT`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    /// `INT.`, `EXT.`, `INT./EXT.`, `I/E`, `EST.` as written, upper-cased; `null` when the heading
    /// has no such prefix.
    pub interior_exterior: Option<String>,
    pub location: String,
    pub time_of_day: Option<String>,
}

/// What a beat is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeatKind {
    Action,
    Dialogue,
    Shot,
    Transition,
    Lyric,
    Centered,
}

/// One action paragraph, one speech, a shot, a transition or centered text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beat {
    pub kind: BeatKind,
    pub text: String,
    /// For dialogue and lyrics: the speaker's name, without extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    /// For dialogue: the full cue as printed (`TOMAS (V.O.)`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cue: Option<String>,
    /// For dialogue: parentheticals in the speech, joined with spaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parenthetical: Option<String>,
    /// Dual dialogue: spoken at the same time as the previous speech.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dual: bool,
}

const INT_EXT: [&str; 9] = ["INT./EXT.", "INT/EXT.", "INT./EXT", "INT/EXT", "I/E.", "I/E", "INT.", "EXT.", "EST."];
const INT_EXT_BARE: [&str; 3] = ["INT", "EXT", "EST"];

/// Split a heading into interior/exterior, location and time of day.
pub fn parse_heading(heading: &str) -> Setting {
    let upper = heading.trim().to_uppercase();
    let mut rest = upper.as_str();
    let mut interior_exterior = None;
    for prefix in INT_EXT.iter().chain(&INT_EXT_BARE) {
        if let Some(after) = rest.strip_prefix(prefix)
            && (after.is_empty() || after.starts_with([' ', '.']))
        {
            interior_exterior = Some((*prefix).to_owned());
            rest = after.trim_start_matches('.').trim_start();
            break;
        }
    }
    let (location, time_of_day) = match rest.rsplit_once(" - ") {
        Some((loc, time)) if !time.trim().is_empty() => (loc.trim().to_owned(), Some(time.trim().to_owned())),
        _ => (rest.trim().trim_end_matches(" -").to_owned(), None),
    };
    Setting { interior_exterior, location, time_of_day }
}

/// The stable id of the scene whose heading is element `heading`: a `[[scene-id: …]]` note
/// among the notes and synopses right after it.
pub fn scene_id(sp: &Screenplay, heading: usize) -> Option<String> {
    sp.elements
        .iter()
        .skip(heading + 1)
        .take_while(|e| matches!(e.kind, ElementKind::Note | ElementKind::Synopsis))
        .filter(|e| e.kind == ElementKind::Note)
        .find_map(|e| e.text.trim().strip_prefix(SCENE_ID_PREFIX).map(|id| id.trim().to_owned()))
        .filter(|id| !id.is_empty())
}

/// Give every scene without a stable id one, as a note right after its heading. `next_id` makes
/// the ids (the app passes a random one). Returns how many were added.
pub fn add_scene_ids(sp: &mut Screenplay, mut next_id: impl FnMut() -> String) -> usize {
    let headings: Vec<usize> = sp.scene_heading_indices().collect();
    let mut added = 0;
    for heading in headings.into_iter().rev() {
        if scene_id(sp, heading).is_none() {
            let note = Element::new(ElementKind::Note, format!("{SCENE_ID_PREFIX} {}", next_id()));
            sp.elements.insert(heading + 1, note);
            added += 1;
        }
    }
    added
}

/// Build the scene list from a script and its layout (from [`scriptcraft_layout::paginate`] on
/// the same script).
pub fn scene_list(sp: &Screenplay, layout: &Layout, source: Option<String>) -> SceneList {
    let headings: Vec<usize> = sp.scene_heading_indices().collect();
    let rows = layout.rows_per_page.max(1);
    let end_row = layout.pages.last().map_or(0, |p| {
        let last = p.lines.iter().map(|l| l.row + 1).max().unwrap_or(0);
        layout.absolute_row(scriptcraft_layout::Position { page: p.number, row: last })
    });
    let mut scenes = Vec::new();
    for (n, &heading) in headings.iter().enumerate() {
        let end = headings.get(n + 1).copied().unwrap_or(sp.elements.len());
        let body = sp.elements.get(heading + 1..end).unwrap_or_default();
        let start_row = layout.starts.get(heading).copied().flatten().map_or(0, |p| layout.absolute_row(p));
        let next_row = headings.get(n + 1).and_then(|h| layout.starts.get(*h).copied().flatten()).map_or(end_row, |p| layout.absolute_row(p));
        let length_rows = next_row.saturating_sub(start_row);
        let length_eighths = u32::try_from((length_rows * 8).div_ceil(rows)).unwrap_or(u32::MAX).max(1);
        let Some(h) = sp.elements.get(heading) else { continue };
        let heading_text = h.display_text();
        scenes.push(Scene {
            index: n + 1,
            number: h.scene_number.clone().unwrap_or_else(|| (n + 1).to_string()),
            id: scene_id(sp, heading),
            setting: parse_heading(&heading_text),
            heading: heading_text,
            page_start: layout.page_of(heading),
            page_end: layout.page_of(end.saturating_sub(1).max(heading)),
            length_eighths,
            synopsis: body.iter().find(|e| e.kind == ElementKind::Synopsis).map(|e| e.text.clone()),
            characters: characters(body),
            action: body.iter().filter(|e| e.kind == ElementKind::Action).map(|e| e.text.as_str()).collect::<Vec<_>>().join("\n\n"),
            beats: beats(body),
            element: heading,
        });
    }
    SceneList {
        format: FORMAT.to_owned(),
        version: VERSION,
        generator: format!("ScriptCraft {}", env!("CARGO_PKG_VERSION")),
        title: sp.title().map(str::to_owned),
        source,
        paper: layout.paper.id().to_owned(),
        page_count: layout.page_count(),
        scenes,
    }
}

fn characters(body: &[Element]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for e in body {
        if let Some(name) = e.character_name()
            && !names.contains(&name)
        {
            names.push(name);
        }
    }
    names
}

fn beats(body: &[Element]) -> Vec<Beat> {
    let mut out: Vec<Beat> = Vec::new();
    let simple = |kind, text: &str| Beat { kind, text: text.to_owned(), character: None, cue: None, parenthetical: None, dual: false };
    let mut speaking = false;
    for e in body {
        match e.kind {
            ElementKind::Character => {
                speaking = true;
                out.push(Beat {
                    kind: BeatKind::Dialogue,
                    text: String::new(),
                    character: e.character_name(),
                    cue: Some(e.display_text()),
                    parenthetical: None,
                    dual: e.dual,
                });
            }
            ElementKind::Dialogue | ElementKind::Parenthetical | ElementKind::Lyric if speaking => {
                let Some(beat) = out.last_mut() else { continue };
                let target = if e.kind == ElementKind::Parenthetical { beat.parenthetical.get_or_insert_with(String::new) } else { &mut beat.text };
                if !target.is_empty() {
                    target.push(if e.kind == ElementKind::Parenthetical { ' ' } else { '\n' });
                }
                target.push_str(&e.text);
                if e.kind == ElementKind::Lyric {
                    beat.kind = BeatKind::Lyric;
                }
            }
            ElementKind::Action | ElementKind::Dialogue | ElementKind::Parenthetical => {
                speaking = false;
                out.push(simple(BeatKind::Action, &e.text));
            }
            ElementKind::Lyric => {
                speaking = false;
                out.push(simple(BeatKind::Lyric, &e.text));
            }
            ElementKind::Shot => {
                speaking = false;
                out.push(simple(BeatKind::Shot, &e.display_text()));
            }
            ElementKind::Transition => {
                speaking = false;
                out.push(simple(BeatKind::Transition, &e.display_text()));
            }
            ElementKind::Centered => {
                speaking = false;
                out.push(simple(BeatKind::Centered, &e.text));
            }
            ElementKind::SceneHeading
            | ElementKind::Section
            | ElementKind::Synopsis
            | ElementKind::Note
            | ElementKind::Boneyard
            | ElementKind::PageBreak => {}
        }
    }
    out
}

/// The scene list as pretty-printed JSON.
pub fn to_json(list: &SceneList) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(list)
}

/// Read a scene list, checking its format and major version.
pub fn from_json(text: &str) -> Result<SceneList, String> {
    let list: SceneList = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if list.format != FORMAT {
        return Err(format!("not a ScriptCraft scene list (format \"{}\")", list.format));
    }
    if list.version != VERSION {
        return Err(format!("scene list version {} is not supported (expected {VERSION})", list.version));
    }
    Ok(list)
}

/// The scene list from Fountain text, on US Letter: the one-call form for tools.
pub fn scene_list_from_fountain(text: &str, source: Option<String>) -> SceneList {
    let sp = scriptcraft_fountain::parse(text);
    let layout = scriptcraft_layout::paginate(&sp, &scriptcraft_layout::PageSetup::default());
    scene_list(&sp, &layout, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_split() {
        let s = parse_heading("int. lighthouse - lamp room - night");
        assert_eq!(s.interior_exterior.as_deref(), Some("INT."));
        assert_eq!(s.location, "LIGHTHOUSE - LAMP ROOM");
        assert_eq!(s.time_of_day.as_deref(), Some("NIGHT"));
        let s = parse_heading("INT./EXT. CAR - MOVING");
        assert_eq!((s.interior_exterior.as_deref(), s.location.as_str()), (Some("INT./EXT."), "CAR"));
        let s = parse_heading("I/E VAN");
        assert_eq!((s.interior_exterior.as_deref(), s.location.as_str(), s.time_of_day), (Some("I/E"), "VAN", None));
        let s = parse_heading("THE DREAM");
        assert_eq!((s.interior_exterior, s.location.as_str()), (None, "THE DREAM"));
    }

    #[test]
    fn scene_ids_are_added_once_and_found() {
        let mut sp = scriptcraft_fountain::parse("INT. A - DAY\n\nx\n\nEXT. B - NIGHT\n\n[[scene-id: keep]]\n\ny\n");
        let mut n = 0;
        let added = add_scene_ids(&mut sp, || {
            n += 1;
            format!("id{n}")
        });
        assert_eq!(added, 1);
        assert_eq!(scene_id(&sp, 0).as_deref(), Some("id1"));
        let second = sp.scene_heading_indices().nth(1).unwrap();
        assert_eq!(scene_id(&sp, second).as_deref(), Some("keep"));
        assert_eq!(add_scene_ids(&mut sp, || panic!("no scene needs an id")), 0);
        // and they survive Fountain
        let again = scriptcraft_fountain::parse(&scriptcraft_fountain::write(&sp));
        assert_eq!(scene_id(&again, 0).as_deref(), Some("id1"));
    }
}
