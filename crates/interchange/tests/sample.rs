//! The scene list of the bundled sample.

use scriptcraft_interchange::{BeatKind, from_json, scene_list_from_fountain, to_json};

const LANTERN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");

#[test]
fn sample_scene_list() {
    let list = scene_list_from_fountain(LANTERN, Some("the-lantern-keeper.fountain".into()));
    assert_eq!(list.title.as_deref(), Some("THE LANTERN KEEPER"));
    assert_eq!(list.scenes.len(), 7);
    let first = &list.scenes[0];
    assert_eq!((first.index, first.number.as_str()), (1, "1"));
    assert_eq!(first.heading, "EXT. NORTH POINT LIGHTHOUSE - NIGHT");
    assert_eq!(first.setting.location, "NORTH POINT LIGHTHOUSE");
    assert!(first.action.starts_with("Rain sheets"));
    let lamp = &list.scenes[1];
    assert_eq!(lamp.characters, ["TOMAS", "MAYA"]);
    let speech = lamp.beats.iter().find(|b| b.kind == BeatKind::Dialogue).unwrap();
    assert_eq!(speech.cue.as_deref(), Some("TOMAS (V.O.)"));
    assert_eq!(speech.parenthetical.as_deref(), Some("(through static)"));
    assert!(lamp.beats.iter().any(|b| b.kind == BeatKind::Shot && b.text == "CLOSE ON THE CHART TABLE"));
    assert!(lamp.beats.iter().any(|b| b.kind == BeatKind::Transition));
    assert!(list.scenes.iter().all(|s| s.length_eighths >= 1 && s.page_start <= s.page_end));
    let eighths: u32 = list.scenes.iter().map(|s| s.length_eighths).sum();
    assert!(eighths as usize >= (list.page_count - 1) * 8, "{eighths} eighths over {} pages", list.page_count);
    let dual: Vec<_> = list.scenes[5].beats.iter().filter(|b| b.dual).collect();
    assert_eq!(dual.len(), 2);

    let json = to_json(&list).unwrap();
    assert!(json.contains("\"format\": \"scriptcraft-scenes\""));
    let back = from_json(&json).unwrap();
    assert_eq!(back.scenes.len(), 7);
    assert_eq!(back.scenes[1].beats, list.scenes[1].beats);
}

#[test]
fn rejects_other_documents() {
    assert!(from_json(r#"{"format":"other","version":1,"generator":"x","paper":"a4","page_count":0,"scenes":[]}"#).is_err());
    assert!(from_json(r#"{"format":"scriptcraft-scenes","version":2,"generator":"x","paper":"a4","page_count":0,"scenes":[]}"#).is_err());
}
