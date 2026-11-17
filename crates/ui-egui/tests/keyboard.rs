//! Drive ScriptCraft by keyboard through egui_kittest, with egui's accessibility check on
//! (every widget needs an accessible name), and assert roles, names, focus and the script.

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use scriptcraft_model::ElementKind;
use scriptcraft_ui_egui::{ScriptApp, Startup};

fn harness(startup: Startup) -> Harness<'static, ScriptApp> {
    Harness::builder()
        .with_size(egui::vec2(1280.0, 860.0))
        .with_accessibility_check(true)
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

fn kinds(app: &ScriptApp) -> Vec<ElementKind> {
    app.document().screenplay().elements.iter().map(|e| e.kind).collect()
}

fn type_text(h: &mut Harness<'_, ScriptApp>, text: &str) {
    h.event(egui::Event::Text(text.into()));
    h.run();
}

fn press(h: &mut Harness<'_, ScriptApp>, key: Key) {
    h.key_press(key);
    h.run();
}

/// The accessible name and role of the focused node.
fn focused(h: &Harness<'_, ScriptApp>) -> (String, egui::accesskit::Role) {
    let node = h.get_by(|n| n.is_focused());
    let n = node.accesskit_node();
    (n.label().unwrap_or_default(), n.role())
}

#[test]
fn heading_enter_then_tab_to_character_and_dialogue() {
    let mut h = harness(Startup::default());
    h.run();
    assert_eq!(focused(&h).0, "Scene Heading 1", "the caret starts in the first scene heading");
    type_text(&mut h, "int. lighthouse - night");
    press(&mut h, Key::Enter);
    assert_eq!(kinds(h.state()), [ElementKind::SceneHeading, ElementKind::Action]);
    assert_eq!(focused(&h).0, "Action");
    press(&mut h, Key::Tab);
    assert_eq!(kinds(h.state())[1], ElementKind::Character);
    assert_eq!(focused(&h), ("Character".to_owned(), egui::accesskit::Role::MultilineTextInput));
    type_text(&mut h, "maya");
    press(&mut h, Key::Enter);
    type_text(&mut h, "Hello?");
    let sp = h.state().document().screenplay();
    assert_eq!(kinds(h.state()), [ElementKind::SceneHeading, ElementKind::Character, ElementKind::Dialogue]);
    assert_eq!(sp.elements[0].text, "INT. LIGHTHOUSE - NIGHT");
    assert_eq!(sp.elements[1].text, "MAYA");
    assert_eq!(sp.elements[2].text, "Hello?");
    assert_eq!(focused(&h).0, "Dialogue");
    // Shift+Tab turns dialogue into a parenthetical; Ctrl+Z undoes it.
    press(&mut h, Key::Tab);
    assert_eq!(kinds(h.state())[2], ElementKind::Parenthetical);
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(kinds(h.state())[2], ElementKind::Dialogue);
}

#[test]
fn element_kind_is_announced_in_a_live_region() {
    let mut h = harness(Startup::default());
    h.run();
    press(&mut h, Key::Tab); // scene heading → shot
    assert_eq!(kinds(h.state()), [ElementKind::Shot]);
    let live = h.get_all_by_label("Shot").filter(|n| n.accesskit_node().live() == egui::accesskit::Live::Polite).count();
    assert_eq!(live, 1, "the status bar's element label is a polite live region");
}

#[test]
fn backspace_at_start_joins_and_arrows_move_between_elements() {
    let mut h = harness(Startup::default());
    h.run();
    type_text(&mut h, "INT. A - DAY");
    press(&mut h, Key::Enter);
    type_text(&mut h, "Rain.");
    press(&mut h, Key::Enter);
    assert_eq!(kinds(h.state()).len(), 3);
    press(&mut h, Key::Backspace); // empty action at the start: taken out
    assert_eq!(kinds(h.state()).len(), 2);
    assert_eq!(focused(&h).0, "Action");
    for _ in 0..5 {
        h.key_press(Key::ArrowLeft);
    }
    h.run();
    press(&mut h, Key::ArrowUp);
    assert_eq!(focused(&h).0, "Scene Heading 1");
}

#[test]
fn character_names_complete_with_enter() {
    let mut h = harness(Startup { sample: true, focus_element: Some(9), ..Startup::default() });
    h.run();
    // Element 9 is "Maya grabs the handset."; end it, Tab to a character cue and type "to".
    press(&mut h, Key::Enter);
    press(&mut h, Key::Tab);
    type_text(&mut h, "to");
    assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, "TOMAS").is_some(), "the suggestion list offers TOMAS");
    press(&mut h, Key::Enter);
    let sp = h.state().document().screenplay();
    assert_eq!(sp.elements[10].text, "TOMAS");
    assert_eq!(sp.elements[11].kind, ElementKind::Dialogue);
    assert_eq!(focused(&h).0, "Dialogue");
}

#[test]
fn page_preview_publishes_page_text() {
    let mut h = harness(Startup { sample: true, ..Startup::default() });
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, Key::P);
    h.run();
    let title = h.get_by_role_and_label(egui::accesskit::Role::Document, "Title page");
    assert!(title.accesskit_node().value().unwrap_or_default().contains("THE LANTERN KEEPER"));
    let page = h.get_by_role_and_label(egui::accesskit::Role::Document, "Page 1");
    assert!(page.accesskit_node().value().unwrap_or_default().contains("EXT. NORTH POINT LIGHTHOUSE - NIGHT"));
}

#[test]
fn navigator_lists_scenes_as_buttons_and_jumps() {
    let mut h = harness(Startup { sample: true, ..Startup::default() });
    h.run();
    let scenes = h
        .get_all_by_role(egui::accesskit::Role::Button)
        .filter(|n| n.accesskit_node().label().is_some_and(|l| l.contains("INT.") || l.contains("EXT.")))
        .count();
    assert_eq!(scenes, 7);
    h.get_by_label_contains("3  EXT. THE GULL - DECK - NIGHT").click();
    h.run();
    h.run();
    assert_eq!(focused(&h).0, "Scene Heading 3");
}

#[test]
fn unsaved_changes_dialog_is_keyboard_cancellable() {
    let mut h = harness(Startup::default());
    h.run();
    type_text(&mut h, "INT. A - DAY");
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::N);
    h.run();
    assert!(h.query_by_label("Save changes?").is_some());
    press(&mut h, Key::Escape);
    h.run();
    assert!(h.query_by_label("Save changes?").is_none());
    assert_eq!(h.state().document().screenplay().elements[0].text, "INT. A - DAY", "Esc cancels: nothing is lost");
}
