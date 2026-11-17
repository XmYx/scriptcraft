//! The scene navigator: every scene with its number and page, then the cast.

use egui::{RichText, Ui};
use scriptcraft_engine::complete;
use scriptcraft_interchange::scene_list;

use crate::app::ScriptApp;
use crate::i18n::{self, tr, trf};

pub fn show(app: &mut ScriptApp, ui: &mut Ui) {
    let list = {
        let sp = app.doc.screenplay().clone();
        let layout = app.doc.layout();
        scene_list(&sp, layout, None)
    };
    let current_scene = app.current().and_then(|i| app.doc.screenplay().scene_of(i));
    ui.add_space(4.0);
    ui.heading(tr("Scenes"));
    let lang = i18n::current();
    let scenes = i18n::I18N.trn(lang, list.scenes.len() as u64, "{n} scene", "{n} scenes");
    let pages = i18n::I18N.trn(lang, list.page_count as u64, "{n} page", "{n} pages");
    ui.label(RichText::new(format!("{scenes} · {pages}")).weak());
    ui.separator();
    let mut go_to = None;
    egui::ScrollArea::vertical().id_salt("navigator-scroll").max_height(ui.available_height() * 0.65).show(ui, |ui| {
        for scene in &list.scenes {
            let selected = current_scene == Some(scene.element);
            let text = format!("{:>3}  {}", scene.number, scene.heading);
            let response = ui.add(egui::Button::selectable(selected, RichText::new(text).monospace()).truncate()).on_hover_text(format!(
                "{}{}",
                trf(
                    "Page {page} · {eighths}/8 of a page",
                    &[("page", &scene.page_start.to_string()), ("eighths", &scene.length_eighths.to_string())]
                ),
                cast_line(&scene.characters)
            ));
            if response.clicked() {
                go_to = Some(scene.element);
            }
        }
        if list.scenes.is_empty() {
            ui.label(RichText::new(tr("No scene headings yet. Start one with INT. or EXT.")).weak());
        }
    });
    ui.separator();
    ui.heading(tr("Characters"));
    let names = complete::character_names(app.doc.screenplay(), None);
    egui::ScrollArea::vertical().id_salt("cast-scroll").show(ui, |ui| {
        for name in names {
            ui.label(RichText::new(name).monospace());
        }
    });
    if let Some(element) = go_to {
        app.jump = Some(element);
        app.page_preview = false;
    }
}

fn cast_line(characters: &[String]) -> String {
    if characters.is_empty() { String::new() } else { format!("\n{}", characters.join(", ")) }
}
