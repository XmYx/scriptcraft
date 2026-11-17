//! Reports: script statistics, characters, locations and scenes.

use scriptcraft_layout::Layout;
use scriptcraft_model::{ElementKind, Screenplay, character_name};

/// Whole-script numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Statistics {
    pub pages: usize,
    pub scenes: usize,
    pub words: usize,
    pub dialogue_words: usize,
    pub speeches: usize,
    pub characters: usize,
    pub locations: usize,
    pub interior_scenes: usize,
    pub exterior_scenes: usize,
    pub day_scenes: usize,
    pub night_scenes: usize,
}

/// One character's numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterRow {
    pub name: String,
    pub speeches: usize,
    pub words: usize,
    /// Scenes the character speaks in.
    pub scenes: usize,
    /// The page of the first speech.
    pub first_page: u32,
}

/// One location's numbers (the heading without its time of day).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationRow {
    pub location: String,
    pub interior_exterior: String,
    pub scenes: usize,
    /// Eighths of a page over all its scenes.
    pub eighths: u32,
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Per-character numbers, most speeches first.
pub fn characters(sp: &Screenplay, layout: &Layout) -> Vec<CharacterRow> {
    let mut rows: Vec<CharacterRow> = Vec::new();
    let mut scenes_seen: Vec<(String, Option<usize>)> = Vec::new();
    let mut speaker: Option<usize> = None;
    for (i, e) in sp.elements.iter().enumerate() {
        match e.kind {
            ElementKind::Character => {
                let name = character_name(&e.text);
                if name.is_empty() {
                    speaker = None;
                    continue;
                }
                let at = match rows.iter().position(|r| r.name == name) {
                    Some(p) => p,
                    None => {
                        rows.push(CharacterRow { name: name.clone(), speeches: 0, words: 0, scenes: 0, first_page: layout.page_of(i) });
                        rows.len() - 1
                    }
                };
                if let Some(r) = rows.get_mut(at) {
                    r.speeches += 1;
                    let scene = sp.scene_of(i);
                    if !scenes_seen.contains(&(name.clone(), scene)) {
                        scenes_seen.push((name, scene));
                        r.scenes += 1;
                    }
                }
                speaker = Some(at);
            }
            ElementKind::Dialogue | ElementKind::Lyric => {
                if let Some(r) = speaker.and_then(|s| rows.get_mut(s)) {
                    r.words += words(&e.text);
                }
            }
            ElementKind::Parenthetical => {}
            _ => speaker = None,
        }
    }
    rows.sort_by(|a, b| b.speeches.cmp(&a.speeches).then_with(|| a.name.cmp(&b.name)));
    rows
}

/// Per-location numbers, most scenes first.
pub fn locations(sp: &Screenplay, layout: &Layout) -> Vec<LocationRow> {
    let list = scriptcraft_interchange::scene_list(sp, layout, None);
    let mut rows: Vec<LocationRow> = Vec::new();
    for scene in &list.scenes {
        let ie = scene.setting.interior_exterior.clone().unwrap_or_default();
        match rows.iter_mut().find(|r| r.location == scene.setting.location && r.interior_exterior == ie) {
            Some(r) => {
                r.scenes += 1;
                r.eighths += scene.length_eighths;
            }
            None => {
                rows.push(LocationRow { location: scene.setting.location.clone(), interior_exterior: ie, scenes: 1, eighths: scene.length_eighths })
            }
        }
    }
    rows.sort_by(|a, b| b.scenes.cmp(&a.scenes).then_with(|| a.location.cmp(&b.location)));
    rows
}

/// Whole-script statistics.
pub fn statistics(sp: &Screenplay, layout: &Layout) -> Statistics {
    let list = scriptcraft_interchange::scene_list(sp, layout, None);
    let mut s = Statistics { pages: layout.page_count(), scenes: list.scenes.len(), ..Statistics::default() };
    let mut in_speech = false;
    for e in sp.elements.iter().filter(|e| e.kind.is_printed()) {
        let w = words(&e.text);
        s.words += w;
        match e.kind {
            ElementKind::Character => {
                s.speeches += 1;
                in_speech = true;
            }
            ElementKind::Dialogue | ElementKind::Lyric if in_speech => s.dialogue_words += w,
            ElementKind::Parenthetical => {}
            _ => in_speech = false,
        }
    }
    s.characters = characters(sp, layout).len();
    s.locations = locations(sp, layout).len();
    for scene in &list.scenes {
        let ie = scene.setting.interior_exterior.as_deref().unwrap_or("");
        if ie.starts_with("INT") || ie.starts_with("I/E") {
            s.interior_scenes += 1;
        }
        if ie.starts_with("EXT") || ie.contains("/EXT") || ie.starts_with("I/E") || ie.starts_with("EST") {
            s.exterior_scenes += 1;
        }
        match scene.setting.time_of_day.as_deref() {
            Some(t) if t.contains("NIGHT") || t.contains("DUSK") || t.contains("EVENING") => s.night_scenes += 1,
            Some(t) if t.contains("DAY") || t.contains("MORNING") || t.contains("DAWN") || t.contains("AFTERNOON") => s.day_scenes += 1,
            _ => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const LANTERN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");

    #[test]
    fn sample_reports() {
        let sp = scriptcraft_fountain::parse(LANTERN);
        let layout = scriptcraft_layout::paginate(&sp, &Default::default());
        let chars = characters(&sp, &layout);
        assert_eq!(chars.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["TOMAS", "MAYA", "ARI"]);
        assert_eq!((chars[0].speeches, chars[1].speeches, chars[2].speeches), (7, 5, 2));
        assert_eq!(chars[2].scenes, 2);
        let locs = locations(&sp, &layout);
        assert_eq!(locs[0].location, "LIGHTHOUSE - LAMP ROOM"); // tied with THE GULL - DECK at 2 scenes; ties sort by name
        assert_eq!(locs[0].scenes, 2);
        let s = statistics(&sp, &layout);
        assert_eq!((s.scenes, s.characters, s.speeches), (7, 3, 14));
        assert_eq!((s.interior_scenes, s.exterior_scenes), (3, 4));
        assert_eq!((s.day_scenes, s.night_scenes), (1, 3));
        assert!(s.dialogue_words > 50 && s.words > s.dialogue_words);
    }
}
