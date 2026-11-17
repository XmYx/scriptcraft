//! Dialogs: unsaved changes, title page, keyboard shortcuts, about.

use egui::{Context, RichText};
use scriptcraft_model::TitlePage;

use crate::i18n::{tr, trf};

/// The answer to "save changes?".
pub enum Unsaved {
    Save,
    Discard,
    Cancel,
}

/// Ask whether to save before replacing or closing the document. `None` while unanswered.
pub fn unsaved_changes(ctx: &Context, name: &str) -> Option<Unsaved> {
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique("unsaved-changes")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.heading(tr("Save changes?"));
        ui.label(trf("“{name}” has changes that are not saved.", &[("name", name)]));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(tr("Save")).clicked() {
                answer = Some(Unsaved::Save);
            }
            if ui.button(tr("Don't Save")).clicked() {
                answer = Some(Unsaved::Discard);
            }
            if ui.button(tr("Cancel")).clicked() {
                answer = Some(Unsaved::Cancel);
            }
        });
    });
    if modal.should_close() && answer.is_none() {
        answer = Some(Unsaved::Cancel);
    }
    answer
}

/// Offer an autosave newer than the file. `Some(true)` recovers, `Some(false)` declines it,
/// `None` while unanswered.
pub fn recover(ctx: &Context, name: &str) -> Option<bool> {
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique("recover-autosave")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading(tr("Recover unsaved changes?"));
        ui.label(trf("ScriptCraft kept changes to “{name}” that were not saved when it last closed.", &[("name", name)]));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let recover = ui.button(tr("Recover"));
            if ui.memory(|m| m.focused().is_none()) {
                recover.request_focus();
            }
            if recover.clicked() {
                answer = Some(true);
            }
            if ui.button(tr("Don't Recover")).clicked() {
                answer = Some(false);
            }
        });
    });
    if modal.should_close() && answer.is_none() {
        answer = Some(false);
    }
    answer
}

/// The title page editor.
pub struct TitlePageDialog {
    fields: Vec<(String, String)>,
}

impl TitlePageDialog {
    pub fn new(tp: &TitlePage) -> TitlePageDialog {
        let mut fields: Vec<(String, String)> =
            TitlePage::STANDARD_KEYS.iter().map(|k| ((*k).to_owned(), tp.get(k).unwrap_or("").to_owned())).collect();
        for f in &tp.fields {
            if !fields.iter().any(|(k, _)| k.eq_ignore_ascii_case(&f.key)) {
                fields.push((f.key.clone(), f.value.clone()));
            }
        }
        TitlePageDialog { fields }
    }

    /// `Some(Some(page))` on OK, `Some(None)` on Cancel, `None` while open.
    pub fn show(&mut self, ctx: &Context) -> Option<Option<TitlePage>> {
        let mut result = None;
        let modal = egui::Modal::new(egui::Id::unique("title-page")).show(ctx, |ui| {
            ui.set_width(460.0);
            ui.heading(tr("Title Page"));
            egui::Grid::new("title-fields").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                for (key, value) in &mut self.fields {
                    // Standard keys are stored in English and shown translated.
                    let shown = TitlePage::STANDARD_KEYS.iter().find(|k| **k == key.as_str()).map_or(key.as_str(), |k| tr(k));
                    ui.label(shown);
                    let multiline = matches!(key.as_str(), "Contact" | "Notes" | "Source");
                    if multiline {
                        ui.add(egui::TextEdit::multiline(value).desired_rows(2).desired_width(320.0));
                    } else {
                        ui.add(egui::TextEdit::singleline(value).desired_width(320.0));
                    }
                    ui.end_row();
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(tr("OK")).clicked() {
                    let mut tp = TitlePage::default();
                    for (key, value) in &self.fields {
                        tp.set(key, value.trim());
                    }
                    result = Some(Some(tp));
                }
                if ui.button(tr("Cancel")).clicked() {
                    result = Some(None);
                }
            });
        });
        if modal.should_close() && result.is_none() {
            result = Some(None);
        }
        result
    }
}

const KEYS: [(&str, &str); 23] = [
    ("Enter", "End the element and start the next one (character → dialogue, dialogue → action…)"),
    ("Shift+Enter", "New line inside the element"),
    ("Tab", "Change the element: action → character → transition → scene heading → shot; dialogue ⇄ parenthetical"),
    ("Shift+Tab", "Change the element the other way"),
    ("Backspace at the start", "Join with the element above"),
    ("Shift+↑ / Shift+↓ at an edge", "Select whole elements (Ctrl+C, Ctrl+X, Delete act on them)"),
    ("Ctrl+V", "Paste screenplay text; several paragraphs become elements"),
    ("↑ / ↓ at the start / end", "Move to the element above / below"),
    ("Enter or Tab in a suggestion list", "Accept the suggestion (Esc hides the list)"),
    ("Ctrl+1 … Ctrl+7", "Scene heading, action, character, parenthetical, dialogue, transition, shot"),
    ("Ctrl+N / Ctrl+O", "New / open (Fountain or FDX)"),
    ("Ctrl+S / Ctrl+Shift+S", "Save / save as Fountain"),
    ("Ctrl+Z / Ctrl+Shift+Z", "Undo / redo"),
    ("Ctrl+Shift+P", "Page preview"),
    ("Ctrl+Shift+I", "Index cards (arrows move, Enter opens, Alt+arrows move the scene)"),
    ("Ctrl+F / Ctrl+H", "Find / find and replace"),
    ("F3 / Shift+F3", "Next / previous match"),
    ("Ctrl+P", "Export PDF"),
    ("Alt+↑ / Alt+↓", "Move the scene earlier / later"),
    ("F11", "Focus mode (Esc leaves it)"),
    ("F7", "Next misspelled word, with suggestions"),
    ("Ctrl+= / Ctrl+-", "Zoom in / out"),
    ("macOS", "Cmd in place of Ctrl"),
];

pub fn keyboard(ctx: &Context, open: &mut bool) {
    egui::Window::new(tr("Keyboard Shortcuts")).open(open).collapsible(false).resizable(false).show(ctx, |ui| {
        egui::Grid::new("keys").num_columns(2).striped(true).spacing([16.0, 4.0]).show(ui, |ui| {
            for (key, what) in KEYS {
                ui.label(RichText::new(tr(key)).strong());
                ui.label(tr(what));
                ui.end_row();
            }
        });
    });
}

pub fn about(ctx: &Context, open: &mut bool, font_embedded: bool) {
    egui::Window::new(tr("About ScriptCraft")).open(open).collapsible(false).resizable(false).show(ctx, |ui| {
        ui.heading("ScriptCraft");
        ui.label(trf("Version {version}", &[("version", env!("CARGO_PKG_VERSION"))]));
        if let (Some(sha), Some(date)) = (option_env!("SCRIPTCRAFT_BUILD_SHA"), option_env!("SCRIPTCRAFT_BUILD_DATE")) {
            ui.label(trf("Build {sha} ({date})", &[("sha", sha.get(..10).unwrap_or(sha)), ("date", date)]));
        }
        ui.label(tr("Screenwriting for the Crafting Apps. Open source, pure Rust."));
        ui.label(tr("Opens and saves Fountain (.fountain); opens FDX files."));
        ui.label(tr("Licensed under MIT OR Apache-2.0."));
        ui.add_space(6.0);
        ui.label(if font_embedded {
            tr("Script font: Liberation Mono (SIL Open Font License 1.1), from craft-fonts.")
        } else {
            tr("Script font: egui's built-in monospace face (build with CRAFT_FONTS_DIR for Liberation Mono).")
        });
        ui.hyperlink_to(tr("Source code"), "https://github.com/storytold/scriptcraft");
    });
}
