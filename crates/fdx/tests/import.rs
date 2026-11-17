//! FDX import over a fixture written for ScriptCraft.

use scriptcraft_model::ElementKind as K;

const FIXTURE: &[u8] = include_bytes!("fixtures/short-scene.fdx");

#[test]
fn imports_elements_title_page_and_dual_dialogue() {
    let (sp, report) = scriptcraft_fdx::import(FIXTURE).unwrap();
    assert!(report.unknown_types.is_empty());
    let kinds: Vec<K> = sp.elements.iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        [
            K::SceneHeading,
            K::Action,
            K::Character,
            K::Parenthetical,
            K::Dialogue,
            K::Character,
            K::Dialogue,
            K::Character,
            K::Dialogue,
            K::Shot,
            K::Transition,
            K::SceneHeading,
            K::Action
        ]
    );
    assert_eq!(sp.elements[0].text, "INT. LIGHTHOUSE - LAMP ROOM - NIGHT");
    assert_eq!(sp.elements[0].scene_number.as_deref(), Some("1"));
    assert_eq!(sp.elements[1].text, "The lamp turns. Maya trims the wick with brass scissors.");
    assert!(!sp.elements[5].dual);
    assert!(sp.elements[7].dual);
    assert_eq!(sp.elements[12].text, "Rain & wind. Then quiet.");
    assert_eq!(sp.title(), Some("THE LANTERN KEEPER"));
    assert_eq!(sp.title_page.get("Credit"), Some("Written by"));
    assert_eq!(sp.title_page.get("Author"), Some("The ScriptCraft contributors"));
    assert_eq!(sp.title_page.get("Contact"), Some("A test fixture"));
    assert_eq!(sp.title_page.get("Draft date"), Some("Draft 2"));
}

#[test]
fn imported_script_round_trips_through_fountain() {
    let (sp, _) = scriptcraft_fdx::import(FIXTURE).unwrap();
    let text = scriptcraft_fountain::write(&sp);
    assert_eq!(scriptcraft_fountain::parse(&text), sp, "{text}");
}

#[test]
fn truncated_and_corrupt_files_are_errors_not_panics() {
    for cut in (0..FIXTURE.len()).step_by(7) {
        let _ = scriptcraft_fdx::import(&FIXTURE[..cut]);
    }
    let mut flipped = FIXTURE.to_vec();
    for i in (0..flipped.len()).step_by(13) {
        flipped[i] ^= 0x20;
        let _ = scriptcraft_fdx::import(&flipped);
    }
}

#[test]
fn export_round_trips_everything_import_reads() {
    let (sp, _) = scriptcraft_fdx::import(FIXTURE).unwrap();
    let (xml, report) = scriptcraft_fdx::export(&sp);
    assert_eq!(report.left_out, 0);
    let (back, _) = scriptcraft_fdx::import(xml.as_bytes()).unwrap();
    assert_eq!(back, sp, "{xml}");
}

#[test]
fn sample_script_survives_fdx() {
    let sp = scriptcraft_fountain::parse(include_str!("../../../assets/samples/the-lantern-keeper.fountain"));
    let (xml, report) = scriptcraft_fdx::export(&sp);
    assert_eq!(report.left_out, 4, "section, synopsis, note and omitted text");
    let (back, _) = scriptcraft_fdx::import(xml.as_bytes()).unwrap();
    // FDX marks only the second speech of dual dialogue; the sample marks both.
    let mut printed: Vec<_> = sp.elements.iter().filter(|e| e.kind.is_printed()).cloned().collect();
    if let Some(ari) = printed.iter_mut().find(|e| e.text == "ARI" && e.dual) {
        ari.dual = false;
    }
    assert_eq!(back.elements, printed);
    assert_eq!(back.title(), sp.title());
    assert!(xml.contains("<DualDialogue>"));
}
