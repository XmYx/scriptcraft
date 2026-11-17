//! The application: state, commands, menus, toolbar and status bar.

use std::path::{Path, PathBuf};

use egui::{Context, Key, KeyboardShortcut, Modifiers, Ui};
use scriptcraft_engine::io::FileKind;
use scriptcraft_engine::{Command, Document, EngineError, io, typing};
use scriptcraft_layout::Paper;
use scriptcraft_model::ElementKind;

use crate::dialogs::{self, TitlePageDialog};
use crate::editor::{self, EditorState};
use crate::i18n::{self, tr, tr_command, trf};

/// The bundled sample screenplay (original work, MIT OR Apache-2.0).
pub const SAMPLE_FOUNTAIN: &str = include_str!("../../../assets/samples/the-lantern-keeper.fountain");

/// How the app starts.
#[derive(Clone, Debug, Default)]
pub struct Startup {
    /// A Fountain or FDX file to open.
    pub file: Option<PathBuf>,
    /// Open the sample script instead of an empty one.
    pub sample: bool,
    /// Start in page preview.
    pub preview: bool,
    pub paper: Paper,
    /// Put the caret in this element at start (screenshots, tests).
    pub focus_element: Option<usize>,
}

/// An action waiting for the answer to "save changes?".
#[derive(Clone, Debug, PartialEq)]
enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    OpenSample,
    Quit,
}

pub struct ScriptApp {
    pub(crate) doc: Document,
    pub(crate) editor: EditorState,
    pub(crate) zoom: f32,
    pub(crate) show_navigator: bool,
    pub(crate) page_preview: bool,
    /// A scene the navigator asked for; focused on the next frame, after the click that chose it.
    pub(crate) jump: Option<usize>,
    status: String,
    pending: Option<Pending>,
    title_dialog: Option<TitlePageDialog>,
    about_open: bool,
    keys_open: bool,
    allow_close: bool,
    quit_requested: bool,
    last_title: String,
    pub(crate) font_embedded: bool,
}

/// The localized name of an element kind.
pub(crate) fn kind_label(kind: ElementKind) -> &'static str {
    tr(kind.label())
}

/// The localized name of a paper size.
pub(crate) fn paper_label(paper: Paper) -> &'static str {
    tr(paper.label())
}

/// A file error in the interface language. The operating system's own reason stays as it gives it.
pub(crate) fn error_text(e: &EngineError) -> String {
    let path = |p: &Path| p.display().to_string();
    match e {
        EngineError::Read { path: p, source } => trf("Could not read {path}: {reason}", &[("path", &path(p)), ("reason", &source.to_string())]),
        EngineError::Write { path: p, source } => trf("Could not save {path}: {reason}", &[("path", &path(p)), ("reason", &source.to_string())]),
        EngineError::TooLarge { path: p, .. } => trf("{path} is larger than 64 MiB and was not opened", &[("path", &path(p))]),
        EngineError::Fountain(e) => trf("This Fountain file was not opened: {reason}", &[("reason", &e.to_string())]),
        EngineError::Fdx { path: p, source } => {
            trf("{path} is not a readable FDX file: {reason}", &[("path", &path(p)), ("reason", &source.to_string())])
        }
        EngineError::SceneList(reason) => trf("Could not write the scene list: {reason}", &[("reason", reason)]),
        EngineError::NoPath => tr("This script has not been saved yet; choose Save As").to_owned(),
    }
}

impl ScriptApp {
    pub fn new(ctx: &Context, startup: Startup) -> ScriptApp {
        i18n::use_system_language();
        let font_embedded = crate::fonts::install(ctx);
        let mut status = String::new();
        let mut doc = if let Some(path) = &startup.file {
            match Document::open(path) {
                Ok((doc, kind, unknown)) => {
                    status = opened_message(path, kind, &unknown);
                    doc
                }
                Err(e) => {
                    status = error_text(&e);
                    Document::new()
                }
            }
        } else if startup.sample {
            sample_document()
        } else {
            Document::new()
        };
        doc.setup.paper = startup.paper;
        let mut editor = EditorState::new(doc.screenplay().elements.len());
        editor.focus_at(startup.focus_element.unwrap_or(0), usize::MAX);
        ScriptApp {
            doc,
            editor,
            zoom: 1.0,
            show_navigator: true,
            page_preview: startup.preview,
            jump: None,
            status,
            pending: None,
            title_dialog: None,
            about_open: false,
            keys_open: false,
            allow_close: false,
            quit_requested: false,
            last_title: String::new(),
            font_embedded,
        }
    }

    /// The open document (for tests and the screenshot driver).
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// The text in the status bar's message area.
    pub fn status(&self) -> &str {
        &self.status
    }

    fn document_name(&self) -> String {
        self.doc.name().unwrap_or_else(|| tr("Untitled").to_owned())
    }

    /// Per-frame work before drawing: shortcuts, window title, close requests.
    pub fn logic(&mut self, ctx: &Context) {
        if let Some(element) = self.jump.take() {
            self.editor.go_to(element);
            ctx.request_repaint();
        }
        self.shortcuts(ctx);
        let title = format!("{}{} — ScriptCraft", self.document_name(), if self.doc.is_dirty() { " •" } else { "" });
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close && self.doc.is_dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }
        // Files dropped on the window open like File ▸ Open.
        let dropped: Option<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).find(|p| !p.as_os_str().is_empty()));
        if let Some(path) = dropped {
            self.guard(Pending::OpenPath(path));
        }
    }

    /// Draw the whole window.
    pub fn ui(&mut self, ui: &mut Ui) {
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        if self.show_navigator {
            egui::Panel::left("navigator").resizable(true).default_size(280.0).min_size(180.0).show(ui, |ui| crate::navigator::show(self, ui));
        }
        egui::CentralPanel::no_frame().show(ui, |ui| {
            if self.page_preview {
                crate::preview::show(self, ui);
            } else {
                editor::show(self, ui);
            }
        });
        self.dialogs(ui.ctx());
    }

    pub(crate) fn set_status(&mut self, text: impl Into<String>) {
        self.status = text.into();
    }

    /// The element the caret is in.
    pub(crate) fn current(&self) -> Option<usize> {
        self.editor.focused.filter(|i| *i < self.doc.screenplay().elements.len())
    }

    fn shortcut(command: Command) -> Option<KeyboardShortcut> {
        let cmd = Modifiers::COMMAND;
        Some(match command {
            Command::SaveAs => KeyboardShortcut::new(cmd | Modifiers::SHIFT, Key::S),
            Command::Redo => KeyboardShortcut::new(cmd | Modifiers::SHIFT, Key::Z),
            Command::TogglePagePreview => KeyboardShortcut::new(cmd | Modifiers::SHIFT, Key::P),
            Command::Save => KeyboardShortcut::new(cmd, Key::S),
            Command::Open => KeyboardShortcut::new(cmd, Key::O),
            Command::New => KeyboardShortcut::new(cmd, Key::N),
            Command::Undo => KeyboardShortcut::new(cmd, Key::Z),
            Command::ZoomIn => KeyboardShortcut::new(cmd, Key::Equals),
            Command::ZoomOut => KeyboardShortcut::new(cmd, Key::Minus),
            Command::SetKind(kind) => {
                let i = ElementKind::WRITING.iter().position(|k| *k == kind)?;
                KeyboardShortcut::new(cmd, DIGITS.get(i).copied()?)
            }
            _ => return None,
        })
    }

    fn shortcuts(&mut self, ctx: &Context) {
        // Most specific first: egui ignores an extra Shift when matching.
        let mut table: Vec<(KeyboardShortcut, Command)> = Command::all().into_iter().filter_map(|c| Some((Self::shortcut(c)?, c))).collect();
        table.sort_by_key(|(s, _)| std::cmp::Reverse(s.modifiers.shift));
        table.push((KeyboardShortcut::new(Modifiers::COMMAND, Key::Y), Command::Redo));
        table.push((KeyboardShortcut::new(Modifiers::COMMAND, Key::Plus), Command::ZoomIn));
        for (shortcut, command) in table {
            if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                self.run(ctx, command);
            }
        }
    }

    /// Run a command.
    pub fn run(&mut self, ctx: &Context, command: Command) {
        match command {
            Command::New => self.guard(Pending::New),
            Command::Open => self.guard(Pending::Open),
            Command::OpenSample => self.guard(Pending::OpenSample),
            Command::Save => self.save(false),
            Command::SaveAs => self.save(true),
            Command::ExportSceneList => self.export_scene_list(),
            Command::ExportText => self.export_text(),
            Command::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Command::Undo => {
                if self.doc.undo() {
                    self.editor.reset_ids(self.doc.screenplay().elements.len());
                    self.editor.refocus();
                } else {
                    self.set_status(tr("Nothing to undo"));
                }
            }
            Command::Redo => {
                if self.doc.redo() {
                    self.editor.reset_ids(self.doc.screenplay().elements.len());
                    self.editor.refocus();
                } else {
                    self.set_status(tr("Nothing to redo"));
                }
            }
            Command::SetKind(kind) => {
                if let Some(i) = self.current()
                    && let Some(caret) = self.doc.edit(|sp| typing::set_kind(sp, i, kind))
                {
                    self.editor.focus_at(caret.element, caret.char_index);
                }
            }
            Command::EditTitlePage => self.title_dialog = Some(TitlePageDialog::new(&self.doc.screenplay().title_page)),
            Command::AddSceneIds => {
                let added = self.doc.edit(|sp| scriptcraft_interchange::add_scene_ids(sp, scriptcraft_engine::new_scene_id));
                self.editor.reset_ids(self.doc.screenplay().elements.len());
                self.set_status(if added == 0 {
                    tr("Every scene already has a stable ID").to_owned()
                } else {
                    i18n::I18N.trn(i18n::current(), added as u64, "Added a stable ID to {n} scene", "Added stable IDs to {n} scenes")
                });
            }
            Command::PaperUsLetter => self.doc.setup.paper = Paper::UsLetter,
            Command::PaperA4 => self.doc.setup.paper = Paper::A4,
            Command::ToggleSceneNumbers => self.doc.setup.scene_numbers = !self.doc.setup.scene_numbers,
            Command::ToggleNavigator => self.show_navigator = !self.show_navigator,
            Command::TogglePagePreview => self.page_preview = !self.page_preview,
            Command::ZoomIn => self.zoom = (self.zoom + 0.1).min(3.0),
            Command::ZoomOut => self.zoom = (self.zoom - 0.1).max(0.6),
            Command::KeyboardHelp => self.keys_open = true,
            Command::About => self.about_open = true,
        }
    }

    /// Ask about unsaved changes before an action that replaces the document.
    fn guard(&mut self, action: Pending) {
        if self.doc.is_dirty() {
            self.pending = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: Pending) {
        match action {
            Pending::New => self.replace_document(Document::new(), tr("New script").to_owned()),
            Pending::OpenSample => self.replace_document(sample_document(), tr("Opened the sample script").to_owned()),
            Pending::Open => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let picked = rfd::FileDialog::new()
                        .set_title(tr("Open Script"))
                        .add_filter(tr("Screenplays (Fountain, FDX)"), &["fountain", "spmd", "txt", "fdx"])
                        .add_filter(tr("All files"), &["*"])
                        .pick_file();
                    if let Some(path) = picked {
                        self.open_path(&path);
                    }
                }
            }
            Pending::OpenPath(path) => self.open_path(&path),
            Pending::Quit => {
                self.allow_close = true;
                self.pending = None;
                self.quit_requested = true;
            }
        }
    }

    fn open_path(&mut self, path: &Path) {
        match Document::open(path) {
            Ok((doc, kind, unknown)) => {
                let status = opened_message(path, kind, &unknown);
                self.replace_document(doc, status);
            }
            Err(e) => self.set_status(error_text(&e)),
        }
    }

    fn replace_document(&mut self, mut doc: Document, status: String) {
        doc.setup = self.doc.setup;
        self.doc = doc;
        self.editor = EditorState::new(self.doc.screenplay().elements.len());
        self.editor.focus_at(0, 0);
        self.set_status(status);
    }

    fn save(&mut self, choose: bool) {
        if choose || self.doc.path.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let name = format!("{}.fountain", self.doc.screenplay().title().map_or_else(|| tr("Untitled").to_owned(), file_stem));
                let Some(path) =
                    rfd::FileDialog::new().set_title(tr("Save Script")).add_filter("Fountain", &["fountain"]).set_file_name(name).save_file()
                else {
                    return;
                };
                match self.doc.save_as(&path) {
                    Ok(()) => self.set_status(trf("Saved {path}", &[("path", &path.display().to_string())])),
                    Err(e) => self.set_status(error_text(&e)),
                }
            }
            return;
        }
        match self.doc.save() {
            Ok(()) => self.set_status(tr("Saved")),
            Err(e) => self.set_status(error_text(&e)),
        }
    }

    fn export_scene_list(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let stem = self.doc.path.as_ref().and_then(|p| p.file_stem()).map_or_else(|| "script".to_owned(), |s| s.to_string_lossy().into_owned());
            let Some(path) = rfd::FileDialog::new()
                .set_title(tr_command("file.export_scene_list", "Export Scene List for Storyboards…"))
                .add_filter(tr("Scene list (JSON)"), &["json"])
                .set_file_name(format!("{stem}.scenes.json"))
                .save_file()
            else {
                return;
            };
            let source = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned());
            let result =
                io::scene_list_json(self.doc.screenplay(), &self.doc.setup, source).and_then(|json| io::write_atomic(&path, json.as_bytes()));
            match result {
                Ok(()) => self.set_status(trf("Exported the scene list to {path}", &[("path", &path.display().to_string())])),
                Err(e) => self.set_status(error_text(&e)),
            }
        }
    }

    fn export_text(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = rfd::FileDialog::new()
                .set_title(tr_command("file.export_text", "Export as Plain Text…"))
                .add_filter(tr("Text"), &["txt"])
                .set_file_name("script.txt")
                .save_file()
            else {
                return;
            };
            let text = io::plain_text(self.doc.screenplay(), &self.doc.setup);
            match io::write_atomic(&path, text.as_bytes()) {
                Ok(()) => self.set_status(trf("Exported {path}", &[("path", &path.display().to_string())])),
                Err(e) => self.set_status(error_text(&e)),
            }
        }
    }

    fn menu_item(&mut self, ui: &mut Ui, command: Command) {
        let mut button = egui::Button::new(tr_command(&command.id(), command.label()));
        if let Some(shortcut) = Self::shortcut(command) {
            button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
        }
        let enabled = match command {
            Command::Undo => self.doc.can_undo(),
            Command::Redo => self.doc.can_redo(),
            Command::SetKind(_) => self.current().is_some(),
            _ => true,
        };
        if ui.add_enabled(enabled, button).clicked() {
            let ctx = ui.ctx().clone();
            self.run(&ctx, command);
            ui.close();
        }
    }

    fn checkbox_item(&mut self, ui: &mut Ui, command: Command, checked: bool) {
        if ui.add(egui::Button::selectable(checked, tr_command(&command.id(), command.label()))).clicked() {
            let ctx = ui.ctx().clone();
            self.run(&ctx, command);
            ui.close();
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(tr("File"), |ui| {
                for c in [Command::New, Command::Open, Command::OpenSample, Command::Save, Command::SaveAs] {
                    self.menu_item(ui, c);
                }
                ui.separator();
                for c in [Command::ExportSceneList, Command::ExportText] {
                    self.menu_item(ui, c);
                }
                ui.add_enabled(false, egui::Button::new(tr("Export PDF… (planned)")));
                ui.separator();
                self.menu_item(ui, Command::Quit);
            });
            ui.menu_button(tr("Edit"), |ui| {
                self.menu_item(ui, Command::Undo);
                self.menu_item(ui, Command::Redo);
            });
            ui.menu_button(tr("Format"), |ui| {
                ui.label(tr("Element"));
                for kind in ElementKind::WRITING {
                    self.menu_item(ui, Command::SetKind(kind));
                }
                ui.separator();
                ui.label(tr("Paper"));
                let paper = self.doc.setup.paper;
                self.checkbox_item(ui, Command::PaperUsLetter, paper == Paper::UsLetter);
                self.checkbox_item(ui, Command::PaperA4, paper == Paper::A4);
                ui.separator();
                let numbers = self.doc.setup.scene_numbers;
                self.checkbox_item(ui, Command::ToggleSceneNumbers, numbers);
            });
            ui.menu_button(tr("Script"), |ui| {
                self.menu_item(ui, Command::EditTitlePage);
                self.menu_item(ui, Command::AddSceneIds);
            });
            ui.menu_button(tr("View"), |ui| {
                let (nav, preview) = (self.show_navigator, self.page_preview);
                self.checkbox_item(ui, Command::ToggleNavigator, nav);
                self.checkbox_item(ui, Command::TogglePagePreview, preview);
                ui.separator();
                self.menu_item(ui, Command::ZoomIn);
                self.menu_item(ui, Command::ZoomOut);
                ui.separator();
                ui.menu_button(tr("Language"), |ui| {
                    for (lang, name) in i18n::languages() {
                        if ui.add(egui::Button::selectable(lang == i18n::current(), name)).clicked() {
                            i18n::set_current(lang);
                            ui.close();
                        }
                    }
                });
            });
            ui.menu_button(tr("Help"), |ui| {
                self.menu_item(ui, Command::KeyboardHelp);
                self.menu_item(ui, Command::About);
            });
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let current = self.current();
            let kind = current.and_then(|i| self.doc.screenplay().elements.get(i)).map(|e| e.kind);
            let element_label = ui.label(tr("Element:"));
            let mut chosen = None;
            ui.add_enabled_ui(kind.is_some(), |ui| {
                egui::ComboBox::from_id_salt("element-kind")
                    .width(150.0)
                    .selected_text(kind.map_or("—", kind_label))
                    .show_ui(ui, |ui| {
                        for k in ElementKind::ALL {
                            if ui.add(egui::Button::selectable(Some(k) == kind, kind_label(k))).clicked() {
                                chosen = Some(k);
                            }
                        }
                    })
                    .response
                    .labelled_by(element_label.id)
                    .on_hover_text(tr("The kind of the element the caret is in (Tab changes it)"));
            });
            if let Some(k) = chosen {
                let ctx = ui.ctx().clone();
                self.run(&ctx, Command::SetKind(k));
            }
            ui.separator();
            let paper_text = ui.label(tr("Paper:"));
            let mut paper = self.doc.setup.paper;
            egui::ComboBox::from_id_salt("paper")
                .selected_text(paper_label(paper))
                .show_ui(ui, |ui| {
                    for p in Paper::ALL {
                        ui.selectable_value(&mut paper, p, paper_label(p));
                    }
                })
                .response
                .labelled_by(paper_text.id);
            self.doc.setup.paper = paper;
            ui.separator();
            ui.toggle_value(&mut self.page_preview, tr("Page Preview"));
            ui.toggle_value(&mut self.show_navigator, tr("Navigator"));
            ui.separator();
            if ui.button("−").on_hover_text(tr("Zoom Out")).widget_info_label(tr("Zoom Out")).clicked() {
                self.zoom = (self.zoom - 0.1).max(0.6);
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("+").on_hover_text(tr("Zoom In")).widget_info_label(tr("Zoom In")).clicked() {
                self.zoom = (self.zoom + 0.1).min(3.0);
            }
        });
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let current = self.current();
        let (pages, page, scenes, scene) = {
            let sp_scene = current.and_then(|i| self.doc.screenplay().scene_of(i));
            let scene_count = self.doc.screenplay().scene_heading_indices().count();
            let scene_no = sp_scene.map(|h| self.doc.screenplay().scene_heading_indices().take_while(|i| *i <= h).count());
            let layout = self.doc.layout();
            (layout.page_count(), current.map(|i| layout.page_of(i)), scene_count, scene_no)
        };
        let kind = current.and_then(|i| self.doc.screenplay().elements.get(i)).map(|e| kind_label(e.kind));
        let dash = "–".to_owned();
        ui.horizontal(|ui| {
            ui.label(trf("Page {page} of {pages}", &[("page", &page.map_or(dash.clone(), |p| p.to_string())), ("pages", &pages.to_string())]));
            ui.separator();
            ui.label(trf("Scene {scene} of {scenes}", &[("scene", &scene.map_or(dash.clone(), |s| s.to_string())), ("scenes", &scenes.to_string())]));
            ui.separator();
            // A live region, so a screen reader hears the element change after Tab or Ctrl+1…7.
            let kind_text = ui.label(kind.unwrap_or("–"));
            ui.ctx().accesskit_node_builder(kind_text.id, |node| node.set_live(egui::accesskit::Live::Polite));
            ui.separator();
            ui.label(if self.doc.is_dirty() { tr("Unsaved changes") } else { tr("Saved") });
            if !self.status.is_empty() {
                ui.separator();
                ui.label(&self.status);
            }
        });
    }

    fn dialogs(&mut self, ctx: &Context) {
        if let Some(pending) = self.pending.clone() {
            match dialogs::unsaved_changes(ctx, &self.document_name()) {
                Some(dialogs::Unsaved::Save) => {
                    self.save(false);
                    self.pending = None;
                    if !self.doc.is_dirty() {
                        self.perform(pending);
                    }
                }
                Some(dialogs::Unsaved::Discard) => {
                    self.pending = None;
                    self.perform(pending);
                }
                Some(dialogs::Unsaved::Cancel) => self.pending = None,
                None => {}
            }
        }
        if self.quit_requested {
            self.quit_requested = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(dialog) = &mut self.title_dialog {
            match dialog.show(ctx) {
                Some(Some(title_page)) => {
                    let mut sp = self.doc.screenplay().clone();
                    sp.title_page = title_page;
                    self.doc.replace(sp);
                    self.title_dialog = None;
                }
                Some(None) => self.title_dialog = None,
                None => {}
            }
        }
        if self.about_open {
            dialogs::about(ctx, &mut self.about_open, self.font_embedded);
        }
        if self.keys_open {
            dialogs::keyboard(ctx, &mut self.keys_open);
        }
    }
}

const DIGITS: [Key; 7] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7];

fn opened_message(path: &Path, kind: FileKind, unknown: &[String]) -> String {
    let path = path.display().to_string();
    match kind {
        FileKind::Fountain => trf("Opened {path}", &[("path", &path)]),
        FileKind::Fdx if unknown.is_empty() => trf("Opened the FDX file {path}. Saving writes Fountain.", &[("path", &path)]),
        FileKind::Fdx => {
            trf("Opened the FDX file {path}. Saving writes Fountain. Read as action: {types}.", &[("path", &path), ("types", &unknown.join(", "))])
        }
    }
}

fn sample_document() -> Document {
    Document::from_screenplay(scriptcraft_fountain::parse(SAMPLE_FOUNTAIN), None)
}

/// A file name from a title: letters, digits, spaces and dashes.
fn file_stem(title: &str) -> String {
    let stem: String = title.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
    let stem = stem.trim();
    if stem.is_empty() { tr("Untitled").to_owned() } else { stem.to_owned() }
}

/// Give a symbol-only button its accessible name.
trait WidgetInfoLabel {
    fn widget_info_label(self, name: &str) -> Self;
}

impl WidgetInfoLabel for egui::Response {
    fn widget_info_label(self, name: &str) -> Self {
        let name = name.to_owned();
        self.ctx.accesskit_node_builder(self.id, |node| node.set_label(name));
        self
    }
}
