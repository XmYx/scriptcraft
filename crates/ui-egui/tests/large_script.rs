//! A feature-length script stays responsive: the editor draws only the lines near the window
//! as text fields (virtualization), and typing and jumping still work at the far end.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use scriptcraft_ui_egui::{ScriptApp, Startup};

fn long_script() -> std::path::PathBuf {
    let mut text = String::new();
    for s in 0..150 {
        text.push_str(&format!(
            "INT. ROOM {s} - DAY\n\nShe crosses the room and looks out at the rain for a long moment before speaking.\n\nMAYA\nWe should go. The tide turns at six and nobody waits for us.\n\nTOMAS\n(quietly)\nThen we go.\n\n"
        ));
    }
    let path = std::env::temp_dir().join(format!("scriptcraft-long-{}.fountain", std::process::id()));
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn long_scripts_draw_only_nearby_lines_and_stay_editable() {
    let path = long_script();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 900.0))
        .with_max_steps(16)
        .with_context_setup(|ctx| {
            scriptcraft_ui_egui::install_fonts(ctx);
        })
        .build_ui_state(
            |ui, app: &mut ScriptApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            ScriptApp::new(&egui::Context::default(), Startup { file: Some(path.clone()), ..Startup::default() }),
        );
    h.run();
    assert_eq!(h.state().document().screenplay().elements.len(), 1050);
    let fields = h.get_all_by_role(egui::accesskit::Role::MultilineTextInput).count();
    assert!(fields < 120, "{fields} text fields drawn for 1050 elements");
    let t = std::time::Instant::now();
    for _ in 0..10 {
        h.step();
    }
    eprintln!("average frame: {:?}", t.elapsed() / 10);
    // Jump by keyboard to the last scene (far below the drawn lines) with Find, then type there.
    h.key_press_modifiers(Modifiers::COMMAND, Key::F);
    h.run();
    h.event(egui::Event::Text("ROOM 149".into()));
    h.run();
    h.key_press_modifiers(Modifiers::NONE, Key::Enter);
    h.run();
    let focused = h.get_by(|n| n.is_focused()).accesskit_node().label().unwrap_or_default();
    assert_eq!(focused, "Scene Heading 150");
    h.key_press_modifiers(Modifiers::NONE, Key::End);
    h.run();
    h.event(egui::Event::Text(" - LATER".into()));
    h.run();
    let sp = h.state().document().screenplay();
    let last = sp.scene_heading_indices().last().unwrap();
    assert_eq!(sp.elements[last].text, "INT. ROOM 149 - DAY - LATER");
    let _ = std::fs::remove_file(path);
}
