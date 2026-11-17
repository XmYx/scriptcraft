//! Fountain round trips over fixtures written for ScriptCraft (no third-party scripts).

use scriptcraft_fountain::{parse, write};
use scriptcraft_model::ElementKind as K;

const LANTERN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");
const EDGES: &str = include_str!("fixtures/edge-cases.fountain");

fn kinds(text: &str) -> Vec<K> {
    parse(text).elements.iter().map(|e| e.kind).collect()
}

#[test]
fn sample_reads_as_expected() {
    let sp = parse(LANTERN);
    assert_eq!(sp.title(), Some("THE LANTERN KEEPER"));
    assert_eq!(sp.title_page.get("Contact").map(|c| c.lines().count()), Some(2));
    let headings: Vec<&str> = sp.elements.iter().filter(|e| e.kind == K::SceneHeading).map(|e| e.text.as_str()).collect();
    assert_eq!(headings.len(), 7);
    assert_eq!(headings[0], "EXT. NORTH POINT LIGHTHOUSE - NIGHT");
    let count = |k: K| sp.elements.iter().filter(|e| e.kind == k).count();
    assert_eq!(count(K::Shot), 1);
    assert_eq!(count(K::Transition), 1); // CUT TO:  (FADE IN: and FADE OUT. are action)
    assert_eq!(count(K::Centered), 1);
    assert_eq!(count(K::Note), 1);
    assert_eq!(count(K::Boneyard), 1);
    assert_eq!(count(K::Section), 1);
    assert_eq!(count(K::Synopsis), 1);
    assert_eq!(sp.elements.iter().filter(|e| e.dual).count(), 2);
    assert!(sp.elements.iter().any(|e| e.kind == K::Parenthetical && e.text == "(through static)"));
}

#[test]
fn edge_cases_read_as_expected() {
    let sp = parse(EDGES);
    assert_eq!(sp.title_page.get("Title"), Some("Edge Cases\nFor the Fountain Reader"));
    assert_eq!(
        kinds(EDGES),
        [
            K::SceneHeading, // .FLASHBACK
            K::Action,       // !INT. ...
            K::Character,
            K::Dialogue,
            K::SceneHeading, // INT./EXT. #7#
            K::SceneHeading, // I/E
            K::SceneHeading, // #12A#
            K::Action,       // indented
            K::Character,
            K::Dialogue,
            K::Transition, // >SMASH CUT
            K::Centered,
            K::Lyric,
            K::PageBreak,
            K::Section,
            K::Synopsis,
            K::Note,
            K::Character, // BANG! (the Fountain ambiguity: capitals followed by a line)
            K::Dialogue,
            K::Action,     // ...and then
            K::Shot,       // POV - ...
            K::Transition, // MATCH CUT TO:
        ]
    );
    assert_eq!(sp.elements[0].text, "FLASHBACK");
    assert_eq!(sp.elements[2].text, "McCAFFREY");
    assert_eq!(sp.elements[4].scene_number.as_deref(), Some("7"));
    assert_eq!(sp.elements[6].scene_number.as_deref(), Some("12A"));
    assert_eq!(sp.elements[7].text, "    Indented action keeps its indentation.\n\tSo does a tab.");
    assert_eq!(sp.elements[9].text, "First line.\n\nThird line after a two-space line.");
    assert_eq!(sp.elements[12].text, "Row, row, row the boat\nGently down the stream");
    assert_eq!(sp.elements[14].level, 2);
    assert_eq!(sp.elements[16].text, "A note with a\nline break");
}

#[track_caller]
fn assert_round_trips(source: &str) {
    let first = parse(source);
    let written = write(&first);
    let second = parse(&written);
    assert_eq!(first, second, "model changed after write → read. Written:\n{written}");
    assert_eq!(write(&second), written, "writer is not idempotent");
}

#[test]
fn sample_round_trips() {
    assert_round_trips(LANTERN);
}

#[test]
fn edge_cases_round_trip() {
    assert_round_trips(EDGES);
}

#[test]
fn sample_writes_back_unchanged() {
    // The sample is written in the writer's own canonical style, so it survives byte for byte.
    assert_eq!(write(&parse(LANTERN)), LANTERN);
}

/// Mutation fuzz: truncations, byte flips and spliced markers never panic, and whatever is read
/// still round-trips.
#[test]
fn mutations_never_panic_and_still_round_trip() {
    let base = LANTERN.as_bytes();
    let mut seed: u64 = 0x5c41_97c4_2a11_0f3d;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let markers: [&[u8]; 12] = [b"\n", b"\n\n", b"  \n", b"/*", b"*/", b"[[", b"]]", b"#", b"^", b">", b"<", b"\r"];
    for round in 0..400 {
        let mut bytes = base.to_vec();
        let cut = (next() as usize) % bytes.len();
        if round % 3 == 0 {
            bytes.truncate(cut);
        }
        for _ in 0..8 {
            if bytes.is_empty() {
                break;
            }
            let at = (next() as usize) % bytes.len();
            match next() % 3 {
                0 => bytes[at] = (next() & 0xff) as u8,
                1 => {
                    let m = markers[(next() as usize) % markers.len()];
                    bytes.splice(at..at, m.iter().copied());
                }
                _ => {
                    bytes.remove(at);
                }
            }
        }
        let sp = scriptcraft_fountain::parse_bytes(&bytes).expect("small input");
        let written = write(&sp);
        let again = parse(&written);
        assert_eq!(write(&again), written, "round {round}: writer not idempotent on mutated input");
    }
}
