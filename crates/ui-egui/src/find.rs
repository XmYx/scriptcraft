//! The find and replace bar (Ctrl+F, Ctrl+H). Enter finds the next match and puts the caret on
//! it with the match selected; F3 and Shift+F3 step through matches from the editor; Esc closes.

use egui::{Key, Ui};
use scriptcraft_engine::find::{self, Match};

use crate::app::ScriptApp;
use crate::i18n::{tr, trf};
use crate::keys;

pub struct FindBar {
    pub query: String,
    pub replacement: String,
    pub match_case: bool,
    pub show_replace: bool,
    /// Put keyboard focus in the query field on the next frame.
    pub focus_query: bool,
    /// The match the caret was last put on.
    pub current: Option<Match>,
}

impl FindBar {
    pub fn new(replace: bool) -> FindBar {
        FindBar { query: String::new(), replacement: String::new(), match_case: false, show_replace: replace, focus_query: true, current: None }
    }
}

/// Go to the next (or previous) match from the caret, wrapping around.
pub fn step(app: &mut ScriptApp, backward: bool) {
    let Some(bar) = &app.find else { return };
    let matches = find::find_all(app.doc.screenplay(), &bar.query, bar.match_case);
    let (element, at) = match bar.current {
        Some(m) if !backward => (m.element, m.start + 1),
        Some(m) => (m.element, m.start),
        None => (app.current().unwrap_or(0), 0),
    };
    let found = find::next(&matches, element, at, backward);
    let total = matches.len();
    if let Some(bar) = &mut app.find {
        bar.current = found;
    }
    match found {
        Some(m) => {
            let n = matches.iter().position(|x| *x == m).map_or(0, |p| p + 1);
            app.editor.select(m.element, m.start, m.len);
            app.view = crate::app::View::Draft;
            app.set_status(trf("Match {n} of {total}", &[("n", &n.to_string()), ("total", &total.to_string())]));
        }
        None => app.set_status(tr("No matches")),
    }
}

fn replace_current(app: &mut ScriptApp) {
    let Some(bar) = &app.find else { return };
    let (Some(m), with) = (bar.current, bar.replacement.clone()) else {
        step(app, false);
        return;
    };
    let replaced = app.doc.edit(|sp| find::replace_one(sp, m, &with));
    if replaced && let Some(bar) = &mut app.find {
        // Continue after the replacement.
        bar.current = Some(Match { element: m.element, start: m.start + with.chars().count().saturating_sub(1), len: 0 });
    }
    step(app, false);
}

fn replace_all(app: &mut ScriptApp) {
    let Some(bar) = &app.find else { return };
    let (query, with, case) = (bar.query.clone(), bar.replacement.clone(), bar.match_case);
    let n = app.doc.edit(|sp| find::replace_all(sp, &query, &with, case));
    app.editor.reset_ids(app.doc.screenplay().elements.len());
    app.set_status(crate::i18n::I18N.trn(crate::i18n::current(), n as u64, "Replaced {n} match", "Replaced {n} matches"));
    if let Some(bar) = &mut app.find {
        bar.current = None;
    }
}

pub fn show(app: &mut ScriptApp, ui: &mut Ui) {
    let mut close = false;
    let mut action: Option<fn(&mut ScriptApp)> = None;
    let Some(bar) = &mut app.find else { return };
    ui.horizontal(|ui| {
        let label = ui.label(tr("Find:"));
        // Keys are taken before the field is drawn: a single-line field would use Enter and Esc
        // to give up focus.
        let query_id = egui::Id::unique("scriptcraft-find-query");
        if ui.memory(|m| m.has_focus(query_id)) {
            if keys::take(ui, Key::Enter, false) {
                action = Some(|app| step(app, false));
            } else if keys::take(ui, Key::Enter, true) {
                action = Some(|app| step(app, true));
            } else if keys::take(ui, Key::Escape, false) {
                close = true;
            }
        }
        let query = ui.add(egui::TextEdit::singleline(&mut bar.query).desired_width(220.0).id(query_id)).labelled_by(label.id);
        if bar.focus_query {
            query.request_focus();
            bar.focus_query = false;
        }
        if query.changed() {
            bar.current = None;
        }
        if ui.button(tr("Previous")).clicked() {
            action = Some(|app| step(app, true));
        }
        if ui.button(tr("Next")).clicked() {
            action = Some(|app| step(app, false));
        }
        ui.checkbox(&mut bar.match_case, tr("Match case"));
        if bar.show_replace {
            ui.separator();
            let label = ui.label(tr("Replace with:"));
            ui.add(egui::TextEdit::singleline(&mut bar.replacement).desired_width(180.0).id_salt("find-replacement")).labelled_by(label.id);
            if ui.button(tr("Replace")).clicked() {
                action = Some(replace_current);
            }
            if ui.button(tr("Replace All")).clicked() {
                action = Some(replace_all);
            }
        } else if ui.button(tr("Replace…")).clicked() {
            bar.show_replace = true;
        }
        if ui.button(tr("Close")).clicked() {
            close = true;
        }
    });
    if let Some(action) = action {
        action(app);
    }
    if close {
        app.find = None;
        app.editor.refocus();
    }
}
