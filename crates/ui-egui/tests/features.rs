//! The polish-pass features driven through egui_kittest with the accessibility check on: find,
//! index cards, revision mode, reports, autosave recovery, focus mode and locked scene numbers.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use scriptcraft_model::ElementKind;
use scriptcraft_ui_egui::{ScriptApp, Startup, View};

fn harness(startup: Startup) -> Harness<'static, ScriptApp> {
    Harness::builder()
        .with_size(egui::vec2(1400.0, 900.0))
        .with_accessibility_check(true)
        // Jumps in the virtualized editor settle over a few frames as measured heights replace estimates.
        .with_max_steps(16)
        .with_context_setup(|ctx| {
            scriptcraft_ui_egui::install_fonts(ctx);
        })
        .build_ui_state(
            |ui, app: &mut ScriptApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            ScriptApp::new(&egui::Context::default(), startup),
        )
}

fn sample() -> Startup {
    Startup { sample: true, ..Startup::default() }
}

fn focused(h: &Harness<'_, ScriptApp>) -> String {
    h.get_by(|n| n.is_focused()).accesskit_node().label().unwrap_or_default()
}

fn chord(h: &mut Harness<'_, ScriptApp>, modifiers: Modifiers, key: Key) {
    h.key_press_modifiers(modifiers, key);
    h.run();
}

fn type_text(h: &mut Harness<'_, ScriptApp>, text: &str) {
    h.event(egui::Event::Text(text.into()));
    h.run();
}

fn headings(app: &ScriptApp) -> Vec<String> {
    let sp = app.document().screenplay();
    sp.scene_heading_indices().map(|i| sp.elements[i].text.clone()).collect()
}

#[test]
fn find_by_keyboard_moves_the_caret_to_each_match() {
    let mut h = harness(sample());
    h.run();
    chord(&mut h, Modifiers::COMMAND, Key::F);
    assert_eq!(focused(&h), "Find:", "Ctrl+F puts the caret in the find field");
    type_text(&mut h, "beam");
    chord(&mut h, Modifiers::NONE, Key::Enter);
    h.run();
    assert!(h.state().status().starts_with("Match 1 of 4"), "{}", h.state().status());
    assert_eq!(focused(&h), "Action", "the caret is on the first match, in the script");
    chord(&mut h, Modifiers::NONE, Key::F3);
    h.run();
    assert!(h.state().status().starts_with("Match 2 of 4"), "{}", h.state().status());
    chord(&mut h, Modifiers::SHIFT, Key::F3);
    h.run();
    assert!(h.state().status().starts_with("Match 1 of 4"), "{}", h.state().status());
}

#[test]
fn replace_all_from_the_find_bar() {
    let mut h = harness(sample());
    h.run();
    chord(&mut h, Modifiers::COMMAND, Key::H);
    type_text(&mut h, "Tomas");
    let field =
        h.get_all_by_role(egui::accesskit::Role::TextInput).filter(|n| n.accesskit_node().label().as_deref() == Some("Replace with:")).count();
    assert_eq!(field, 1);
    h.get_all_by_role(egui::accesskit::Role::TextInput)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Replace with:"))
        .expect("the replacement field")
        .click();
    h.run();
    type_text(&mut h, "Tomás");
    h.get_by_label("Replace All").click();
    h.run();
    assert!(h.state().status().starts_with("Replaced"), "{}", h.state().status());
    let sp = h.state().document().screenplay();
    assert!(sp.elements.iter().all(|e| !e.text.contains("Tomas")));
    assert!(sp.elements.iter().any(|e| e.text.contains("Tomás")));
}

#[test]
fn index_cards_by_keyboard_select_move_and_open_scenes() {
    let mut h = harness(sample());
    h.run();
    chord(&mut h, Modifiers::COMMAND | Modifiers::SHIFT, Key::I);
    h.run();
    assert_eq!(h.state().view(), View::Cards);
    assert!(focused(&h).starts_with("Scene 1: EXT. NORTH POINT LIGHTHOUSE"), "{}", focused(&h));
    chord(&mut h, Modifiers::NONE, Key::ArrowRight);
    h.run();
    assert!(focused(&h).starts_with("Scene 2: INT. LIGHTHOUSE - LAMP ROOM"), "{}", focused(&h));
    // Alt+→ moves scene 2 after scene 3; the card keeps the focus at its new place.
    chord(&mut h, Modifiers::ALT, Key::ArrowRight);
    h.run();
    let order = headings(h.state());
    assert_eq!(order[1], "EXT. THE GULL - DECK - NIGHT");
    assert_eq!(order[2], "INT. LIGHTHOUSE - LAMP ROOM - CONTINUOUS");
    assert!(focused(&h).starts_with("Scene 3: INT. LIGHTHOUSE - LAMP ROOM"), "{}", focused(&h));
    // Enter opens that scene in the draft with the caret in its heading.
    chord(&mut h, Modifiers::NONE, Key::Enter);
    h.run();
    h.run();
    assert_eq!(h.state().view(), View::Draft);
    assert_eq!(focused(&h), "Scene Heading 3");
    // Undo puts the scenes back.
    chord(&mut h, Modifiers::COMMAND, Key::Z);
    assert_eq!(headings(h.state())[1], "INT. LIGHTHOUSE - LAMP ROOM - CONTINUOUS");
}

#[test]
fn revision_mode_marks_what_changes() {
    let mut h = harness(sample());
    h.run();
    h.get_by_label("Production").click();
    h.run();
    h.get_by_label("Start Next Revision (Blue)").click();
    h.run();
    assert!(h.query_by_label("Blue revision, marking changes").is_some());
    // The menu gives the keyboard back to the script: the caret is at the end of element 0
    // (the "Act One" section), and typing marks it, nothing else.
    h.run();
    assert_eq!(focused(&h), "Section");
    type_text(&mut h, " SLOWLY");
    let sp = h.state().document().screenplay();
    assert_eq!(sp.elements.iter().filter(|e| e.revision == 1).count(), 1);
    assert_eq!(sp.revisions.level, 1);
}

#[test]
fn reports_window_lists_characters_and_locations() {
    let mut h = harness(sample());
    h.run();
    h.get_by_label("Script").click();
    h.run();
    h.get_by_label("Reports…").click();
    h.run();
    for label in ["Statistics", "Characters", "Locations", "Words of dialogue", "THE GULL - DECK"] {
        assert!(h.query_all_by_label(label).next().is_some(), "{label}");
    }
}

#[test]
fn a_newer_autosave_is_offered_and_recovered_by_keyboard() {
    let dir = std::env::temp_dir().join(format!("scriptcraft-recover-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("draft.fountain");
    std::fs::write(&file, "INT. SAVED - DAY\n").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(30));
    std::fs::write(scriptcraft_engine::autosave::autosave_path(Some(&file)), "INT. UNSAVED - NIGHT\n").unwrap();
    let mut h = harness(Startup { file: Some(file.clone()), ..Startup::default() });
    h.run();
    assert!(h.query_by_label("Recover unsaved changes?").is_some());
    assert_eq!(focused(&h), "Recover", "Recover has the focus");
    chord(&mut h, Modifiers::NONE, Key::Enter);
    h.run();
    assert_eq!(h.state().document().screenplay().elements[0].text, "INT. UNSAVED - NIGHT");
    assert!(h.state().document().is_dirty(), "recovered changes still need saving");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn focus_mode_hides_the_panels_and_esc_leaves_it() {
    let mut h = harness(sample());
    h.run();
    assert!(h.query_by_label("Navigator").is_some());
    chord(&mut h, Modifiers::NONE, Key::F11);
    h.run();
    assert!(h.query_by_label("Navigator").is_none());
    assert!(h.query_by_label("File").is_none());
    chord(&mut h, Modifiers::NONE, Key::Escape);
    h.run();
    assert!(h.query_by_label("File").is_some());
}

#[test]
fn locking_scene_numbers_numbers_every_scene() {
    let mut h = harness(sample());
    h.run();
    h.get_by_label("Production").click();
    h.run();
    h.get_by_label("Lock Scene Numbers").click();
    h.run();
    let sp = h.state().document().screenplay();
    let numbers: Vec<_> = sp.scene_heading_indices().filter_map(|i| sp.elements[i].scene_number.clone()).collect();
    assert_eq!(numbers, ["1", "2", "3", "4", "5", "6", "7"]);
    assert_eq!(sp.elements.iter().filter(|e| e.kind == ElementKind::SceneHeading).count(), 7);
}

#[test]
fn spelling_next_misspelling_suggests_and_add_word_accepts() {
    let mut h = harness(Startup::default());
    h.run();
    type_text(&mut h, "INT. LIGHTHOUSE - NIGHT");
    chord(&mut h, Modifiers::NONE, Key::Enter);
    type_text(&mut h, "The lamp flikkers in the wind.");
    chord(&mut h, Modifiers::NONE, Key::F7);
    h.run();
    let status = h.state().status().to_owned();
    assert!(status.contains("“flikkers”"), "{status}");
    assert_eq!(focused(&h), "Action", "the caret is on the misspelled word");
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Add Word to Dictionary").click();
    h.run();
    assert!(h.state().status().contains("Added “flikkers”"), "{}", h.state().status());
    chord(&mut h, Modifiers::NONE, Key::F7);
    h.run();
    assert_eq!(h.state().status(), "No misspelled words");
}

#[test]
fn whole_elements_select_copy_cut_and_paste_as_fountain() {
    // Element 10 is "Maya grabs the handset."; the caret starts at its end.
    let mut h = harness(Startup { sample: true, focus_element: Some(10), ..Startup::default() });
    h.run();
    chord(&mut h, Modifiers::SHIFT, Key::ArrowDown);
    chord(&mut h, Modifiers::SHIFT, Key::ArrowDown);
    assert_eq!(h.state().status(), "3 elements selected");
    h.event(egui::Event::Copy);
    // The copy command is in the output of the frame that handled the key.
    h.step();
    let copied: Vec<String> = h
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(copied.last().map(String::as_str), Some("Maya grabs the handset.\n\nMAYA\nI read you, Tomas. You're three hours late.\n"));
    h.run();
    assert_eq!(h.state().status(), "Copied 3 elements");
    let before = h.state().document().screenplay().elements.len();
    h.event(egui::Event::Cut);
    h.run();
    assert_eq!(h.state().document().screenplay().elements.len(), before - 3);
    assert_eq!(h.state().status(), "Removed 3 elements");
    // Pasting screenplay text from anywhere becomes elements with their types.
    h.event(egui::Event::Paste("EXT. JETTY - DAWN\n\nGulls.\n\nARI\nMorning!\n".into()));
    h.run();
    let sp = h.state().document().screenplay();
    assert_eq!(h.state().status(), "Pasted 4 elements");
    let kinds: Vec<ElementKind> = sp.elements.iter().map(|e| e.kind).collect();
    let at = sp.elements.iter().position(|e| e.text == "EXT. JETTY - DAWN").expect("pasted heading");
    assert_eq!(kinds[at..at + 4], [ElementKind::SceneHeading, ElementKind::Action, ElementKind::Character, ElementKind::Dialogue]);
    // Undo takes the paste back out.
    chord(&mut h, Modifiers::COMMAND, Key::Z);
    assert!(h.state().document().screenplay().elements.iter().all(|e| e.text != "EXT. JETTY - DAWN"));
}
