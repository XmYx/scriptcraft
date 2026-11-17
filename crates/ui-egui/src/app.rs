//! The application: state, commands, menus, toolbar and status bar.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui::{Context, Key, KeyboardShortcut, Modifiers, Ui};
use scriptcraft_engine::io::FileKind;
use scriptcraft_engine::{Command, Document, EngineError, autosave, io, numbering, revisions, scenes, typing};
use scriptcraft_layout::Paper;
use scriptcraft_model::{ElementKind, revision_colour};

use crate::dialogs::{self, TitlePageDialog};
use crate::editor::{self, EditorState};
use crate::find::FindBar;
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
    /// Start in the index cards.
    pub cards: bool,
    pub paper: Paper,
    /// Put the caret in this element at start (screenshots, tests).
    pub focus_element: Option<usize>,
    /// Start a revision and mark these elements with it (screenshots of revision mode).
    pub demo_revision: Vec<usize>,
    /// Open the find bar with this text (screenshots).
    pub find: Option<String>,
}

/// What the central area shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// The editable draft on one long sheet.
    Draft,
    /// The printed pages, read-only.
    Preview,
    /// One card per scene.
    Cards,
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
    pub(crate) view: View,
    pub(crate) focus_mode: bool,
    pub(crate) typewriter: bool,
    pub(crate) dark_paper: bool,
    pub(crate) spelling: bool,
    /// The word the last Next Misspelling stopped on, for Add Word to Dictionary.
    misspelling: Option<String>,
    pub(crate) find: Option<FindBar>,
    pub(crate) cards: crate::cards::CardsState,
    /// A scene the navigator asked for; focused on the next frame, after the click that chose it.
    pub(crate) jump: Option<usize>,
    refocus_next: bool,
    status: String,
    pending: Option<Pending>,
    recovery: Option<PathBuf>,
    title_dialog: Option<TitlePageDialog>,
    reports_open: bool,
    about_open: bool,
    keys_open: bool,
    allow_close: bool,
    quit_requested: bool,
    last_title: String,
    last_autosave: Instant,
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

/// The localized name of a revision colour (`Blue`).
pub(crate) fn colour_label(level: u8) -> &'static str {
    tr(revision_colour(level))
}

/// "Blue Revision", for headers.
pub(crate) fn revision_label(level: u8) -> String {
    trf("{colour} Revision", &[("colour", colour_label(level))])
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
        EngineError::Pdf(reason) => trf("Could not write the PDF: {reason}", &[("reason", reason)]),
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
        if !startup.demo_revision.is_empty() {
            let marked = startup.demo_revision.clone();
            doc.edit(|sp| {
                revisions::start_next(sp);
                let level = sp.revisions.level;
                for i in marked {
                    if let Some(e) = sp.elements.get_mut(i) {
                        e.revision = level;
                    }
                }
            });
        }
        // A newer autosave than the file (or an untitled one) means the last session ended
        // before saving: offer it back. Not for the bundled sample.
        let recovery = if startup.sample { None } else { autosave::recovery_for(doc.path.as_deref()) };
        let mut editor = EditorState::new(doc.screenplay().elements.len());
        editor.focus_at(startup.focus_element.unwrap_or(0), usize::MAX);
        let view = if startup.cards {
            View::Cards
        } else if startup.preview {
            View::Preview
        } else {
            View::Draft
        };
        let mut app = ScriptApp {
            doc,
            editor,
            zoom: 1.0,
            show_navigator: true,
            view,
            focus_mode: false,
            typewriter: false,
            dark_paper: false,
            spelling: true,
            misspelling: None,
            find: None,
            cards: crate::cards::CardsState::default(),
            jump: None,
            refocus_next: false,
            status,
            pending: None,
            recovery,
            title_dialog: None,
            reports_open: false,
            about_open: false,
            keys_open: false,
            allow_close: false,
            quit_requested: false,
            last_title: String::new(),
            last_autosave: Instant::now(),
            font_embedded,
        };
        if let Some(query) = startup.find {
            let mut bar = FindBar::new(false);
            bar.query = query;
            app.find = Some(bar);
            crate::find::step(&mut app, false);
        }
        app
    }

    /// The open document (for tests and the screenshot driver).
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// The text in the status bar's message area.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// What the central area shows.
    pub fn view(&self) -> View {
        self.view
    }

    fn document_name(&self) -> String {
        self.doc.name().unwrap_or_else(|| tr("Untitled").to_owned())
    }

    /// Per-frame work before drawing: shortcuts, window title, close requests, autosave.
    pub fn logic(&mut self, ctx: &Context) {
        if let Some(element) = self.jump.take() {
            self.editor.go_to(element);
            ctx.request_repaint();
        }
        if std::mem::take(&mut self.refocus_next) {
            self.editor.refocus();
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
        if self.last_autosave.elapsed() >= Duration::from_secs(autosave::AUTOSAVE_SECONDS) {
            self.last_autosave = Instant::now();
            if let Err(e) = self.doc.autosave() {
                self.set_status(trf("Autosave failed: {reason}", &[("reason", &error_text(&e))]));
            }
        }
        if self.doc.is_dirty() {
            ctx.request_repaint_after(Duration::from_secs(autosave::AUTOSAVE_SECONDS));
        }
    }

    /// Draw the whole window.
    pub fn ui(&mut self, ui: &mut Ui) {
        if !self.focus_mode {
            egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
            egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        }
        if self.find.is_some() {
            egui::Panel::top("find").show(ui, |ui| crate::find::show(self, ui));
        }
        if !self.focus_mode {
            egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
            if self.show_navigator && self.view != View::Cards {
                egui::Panel::left("navigator").resizable(true).default_size(280.0).min_size(180.0).show(ui, |ui| crate::navigator::show(self, ui));
            }
        }
        egui::CentralPanel::no_frame().show(ui, |ui| match self.view {
            View::Preview => crate::preview::show(self, ui),
            View::Cards => crate::cards::show(self, ui),
            View::Draft => editor::show(self, ui),
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

    /// The scene (0-based) the caret or the selected card is in.
    pub(crate) fn current_scene(&self) -> Option<usize> {
        if self.view == View::Cards {
            return self.cards.selected;
        }
        let heading = self.current().and_then(|i| self.doc.screenplay().scene_of(i))?;
        self.doc.screenplay().scene_heading_indices().position(|h| h == heading)
    }

    fn shortcut(command: Command) -> Option<KeyboardShortcut> {
        let cmd = Modifiers::COMMAND;
        let shift = Modifiers::SHIFT;
        Some(match command {
            Command::SaveAs => KeyboardShortcut::new(cmd | shift, Key::S),
            Command::Redo => KeyboardShortcut::new(cmd | shift, Key::Z),
            Command::TogglePagePreview => KeyboardShortcut::new(cmd | shift, Key::P),
            Command::ToggleIndexCards => KeyboardShortcut::new(cmd | shift, Key::I),
            Command::Replace => KeyboardShortcut::new(cmd, Key::H),
            Command::ExportPdf => KeyboardShortcut::new(cmd, Key::P),
            Command::Save => KeyboardShortcut::new(cmd, Key::S),
            Command::Open => KeyboardShortcut::new(cmd, Key::O),
            Command::New => KeyboardShortcut::new(cmd, Key::N),
            Command::Undo => KeyboardShortcut::new(cmd, Key::Z),
            Command::Find => KeyboardShortcut::new(cmd, Key::F),
            Command::FindNext => KeyboardShortcut::new(Modifiers::NONE, Key::F3),
            Command::NextMisspelling => KeyboardShortcut::new(Modifiers::NONE, Key::F7),
            Command::FindPrevious => KeyboardShortcut::new(shift, Key::F3),
            Command::ToggleFocusMode => KeyboardShortcut::new(Modifiers::NONE, Key::F11),
            Command::MoveSceneUp => KeyboardShortcut::new(Modifiers::ALT, Key::ArrowUp),
            Command::MoveSceneDown => KeyboardShortcut::new(Modifiers::ALT, Key::ArrowDown),
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
        table.sort_by_key(|(s, _)| std::cmp::Reverse((s.modifiers.shift, s.modifiers.alt)));
        table.push((KeyboardShortcut::new(Modifiers::COMMAND, Key::Y), Command::Redo));
        table.push((KeyboardShortcut::new(Modifiers::COMMAND, Key::Plus), Command::ZoomIn));
        for (shortcut, command) in table {
            if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                self.run(ctx, command);
            }
        }
        if self.focus_mode && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) && self.find.is_none() {
            self.focus_mode = false;
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
            Command::ExportPdf => self.export_pdf(),
            Command::ExportFdx => self.export_fdx(),
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
            Command::Find => self.open_find(false),
            Command::Replace => self.open_find(true),
            Command::FindNext => {
                if self.find.is_none() {
                    self.open_find(false);
                } else {
                    crate::find::step(self, false);
                }
            }
            Command::FindPrevious => crate::find::step(self, true),
            Command::ToggleSpelling => self.spelling = !self.spelling,
            Command::NextMisspelling => self.next_misspelling(),
            Command::AddToDictionary => {
                if let Some(word) = self.misspelling.take() {
                    scriptcraft_engine::spelling::add_word(&word);
                    self.editor.spell_cache.clear();
                    self.set_status(trf("Added “{word}” to the dictionary", &[("word", &word)]));
                } else {
                    self.set_status(tr("Use Next Misspelling (F7) to choose a word first"));
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
            Command::Reports => self.reports_open = true,
            Command::MoveSceneUp | Command::MoveSceneDown => self.move_scene(command == Command::MoveSceneUp),
            Command::LockSceneNumbers => {
                let n = self.doc.edit(numbering::lock);
                self.doc.setup.scene_numbers = true;
                self.set_status(i18n::I18N.trn(
                    i18n::current(),
                    n as u64,
                    "Locked scene numbers; numbered {n} scene",
                    "Locked scene numbers; numbered {n} scenes",
                ));
            }
            Command::UnlockSceneNumbers => {
                self.doc.edit(numbering::unlock);
                self.set_status(tr("Scene numbers unlocked"));
            }
            Command::StartNextRevision => {
                let level = self.doc.edit(revisions::start_next);
                self.set_status(trf("Started the {colour} revision; changes are marked", &[("colour", colour_label(level))]));
            }
            Command::ToggleRevisionTracking => {
                let on = self.doc.edit(|sp| {
                    if sp.revisions.level == 0 {
                        sp.revisions.level = 1;
                    }
                    sp.revisions.tracking = !sp.revisions.tracking;
                    sp.revisions.tracking
                });
                self.set_status(if on { tr("Marking revisions") } else { tr("Not marking revisions") });
            }
            Command::ClearRevisionMarks => {
                let n = self.doc.edit(revisions::clear_marks);
                self.set_status(i18n::I18N.trn(i18n::current(), n as u64, "Cleared {n} revision mark", "Cleared {n} revision marks"));
            }
            Command::ToggleAutoContd => self.doc.setup.auto_contd = !self.doc.setup.auto_contd,
            Command::PaperUsLetter => self.doc.setup.paper = Paper::UsLetter,
            Command::PaperA4 => self.doc.setup.paper = Paper::A4,
            Command::ToggleSceneNumbers => self.doc.setup.scene_numbers = !self.doc.setup.scene_numbers,
            Command::ToggleNavigator => self.show_navigator = !self.show_navigator,
            Command::TogglePagePreview => self.view = if self.view == View::Preview { View::Draft } else { View::Preview },
            Command::ToggleIndexCards => {
                if self.view == View::Cards {
                    self.view = View::Draft;
                    self.editor.refocus();
                } else {
                    self.cards.selected = self.current_scene().or(Some(0));
                    self.cards.focus_selected = true;
                    self.view = View::Cards;
                }
            }
            Command::ToggleFocusMode => self.focus_mode = !self.focus_mode,
            Command::ToggleTypewriter => self.typewriter = !self.typewriter,
            Command::ToggleDarkPaper => self.dark_paper = !self.dark_paper,
            Command::ZoomIn => self.zoom = (self.zoom + 0.1).min(3.0),
            Command::ZoomOut => self.zoom = (self.zoom - 0.1).max(0.6),
            Command::KeyboardHelp => self.keys_open = true,
            Command::About => self.about_open = true,
        }
    }

    /// Select the next misspelled word after the caret and list suggestions.
    fn next_misspelling(&mut self) {
        let (element, at) = match self.current() {
            Some(i) => (i, self.editor.caret_hint.unwrap_or(0)),
            None => (0, 0),
        };
        match scriptcraft_engine::spelling::next(self.doc.screenplay(), element, at) {
            Some(m) => {
                self.view = View::Draft;
                self.editor.select(m.element, m.start, m.len);
                self.editor.caret_hint = Some(m.start);
                let suggestions = scriptcraft_engine::spelling::suggestions(&m.word, 5);
                self.set_status(if suggestions.is_empty() {
                    trf("Not in the dictionary: “{word}”", &[("word", &m.word)])
                } else {
                    trf("Not in the dictionary: “{word}”. Suggestions: {list}", &[("word", &m.word), ("list", &suggestions.join(", "))])
                });
                self.misspelling = Some(m.word);
            }
            None => {
                self.misspelling = None;
                self.set_status(tr("No misspelled words"));
            }
        }
    }

    fn open_find(&mut self, replace: bool) {
        match &mut self.find {
            Some(bar) => {
                bar.show_replace |= replace;
                bar.focus_query = true;
            }
            None => self.find = Some(FindBar::new(replace)),
        }
    }

    /// Move the current scene one place earlier or later (index cards, Alt+↑/↓).
    fn move_scene(&mut self, earlier: bool) {
        let Some(scene) = self.current_scene() else { return };
        let count = self.doc.screenplay().scene_heading_indices().count();
        let to = if earlier { scene.checked_sub(1) } else { Some(scene + 1).filter(|t| *t < count) };
        let Some(to) = to else { return };
        if let Some(heading) = self.doc.edit(|sp| scenes::move_scene(sp, scene, to)) {
            self.editor.reset_ids(self.doc.screenplay().elements.len());
            if self.view == View::Cards {
                self.cards.selected = Some(to);
                self.cards.focus_selected = true;
            } else {
                self.editor.focus_at(heading, 0);
            }
            self.set_status(trf("Moved the scene to position {n}", &[("n", &(to + 1).to_string())]));
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
                autosave::discard(self.doc.path.as_deref());
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
                self.recovery = autosave::recovery_for(doc.path.as_deref());
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
        self.cards = crate::cards::CardsState::default();
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
                let was_untitled = self.doc.path.is_none();
                match self.doc.save_as(&path) {
                    Ok(()) => {
                        if was_untitled {
                            autosave::discard(None);
                        }
                        self.set_status(trf("Saved {path}", &[("path", &path.display().to_string())]));
                    }
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

    fn export_name(&self, extension: &str) -> String {
        let stem = self
            .doc
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .or_else(|| self.doc.screenplay().title().map(file_stem))
            .unwrap_or_else(|| "script".to_owned());
        format!("{stem}.{extension}")
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_dialog(&self, command: Command, filter: &str, extension: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr_command(&command.id(), command.label()))
            .add_filter(filter, &[extension])
            .set_file_name(self.export_name(extension))
            .save_file()
    }

    fn export_scene_list(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = self.save_dialog(Command::ExportSceneList, tr("Scene list (JSON)"), "json") else { return };
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
            let Some(path) = self.save_dialog(Command::ExportText, tr("Text"), "txt") else { return };
            let text = io::plain_text(self.doc.screenplay(), &self.doc.setup);
            match io::write_atomic(&path, text.as_bytes()) {
                Ok(()) => self.set_status(trf("Exported {path}", &[("path", &path.display().to_string())])),
                Err(e) => self.set_status(error_text(&e)),
            }
        }
    }

    fn export_pdf(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = self.save_dialog(Command::ExportPdf, "PDF", "pdf") else { return };
            let result = io::pdf_with(self.doc.screenplay(), &self.doc.setup, revision_label).and_then(|bytes| io::write_atomic(&path, &bytes));
            match result {
                Ok(()) => self.set_status(trf("Exported {path}", &[("path", &path.display().to_string())])),
                Err(e) => self.set_status(error_text(&e)),
            }
        }
    }

    fn export_fdx(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = self.save_dialog(Command::ExportFdx, "FDX", "fdx") else { return };
            match io::save_fdx(&path, self.doc.screenplay()) {
                Ok(0) => self.set_status(trf("Exported {path}", &[("path", &path.display().to_string())])),
                Ok(n) => self.set_status(trf(
                    "Exported {path}; {n} outline and note elements are not part of FDX and were left out",
                    &[("path", &path.display().to_string()), ("n", &n.to_string())],
                )),
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
            Command::MoveSceneUp | Command::MoveSceneDown => self.current_scene().is_some(),
            _ => true,
        };
        if ui.add_enabled(enabled, button).clicked() {
            let ctx = ui.ctx().clone();
            self.run(&ctx, command);
            self.after_menu();
            ui.close();
        }
    }

    /// After a menu command, the keyboard goes back to the script (unless the command opened
    /// something that took it: a dialog, the find bar or the cards).
    fn after_menu(&mut self) {
        let dialog = self.title_dialog.is_some() || self.reports_open || self.about_open || self.keys_open || self.pending.is_some();
        if self.view == View::Draft && self.find.is_none() && !dialog {
            // On the next frame: closing the menu this frame would take the focus away again.
            self.refocus_next = true;
        }
    }

    fn checkbox_item(&mut self, ui: &mut Ui, command: Command, checked: bool) {
        let mut button = egui::Button::selectable(checked, tr_command(&command.id(), command.label()));
        if let Some(shortcut) = Self::shortcut(command) {
            button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
        }
        if ui.add(button).clicked() {
            let ctx = ui.ctx().clone();
            self.run(&ctx, command);
            self.after_menu();
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
                for c in [Command::ExportPdf, Command::ExportFdx, Command::ExportSceneList, Command::ExportText] {
                    self.menu_item(ui, c);
                }
                ui.separator();
                self.menu_item(ui, Command::Quit);
            });
            ui.menu_button(tr("Edit"), |ui| {
                for c in [Command::Undo, Command::Redo] {
                    self.menu_item(ui, c);
                }
                ui.separator();
                for c in [Command::Find, Command::Replace, Command::FindNext, Command::FindPrevious] {
                    self.menu_item(ui, c);
                }
                ui.separator();
                let spelling = self.spelling;
                self.checkbox_item(ui, Command::ToggleSpelling, spelling);
                self.menu_item(ui, Command::NextMisspelling);
                self.menu_item(ui, Command::AddToDictionary);
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
                let (numbers, contd) = (self.doc.setup.scene_numbers, self.doc.setup.auto_contd);
                self.checkbox_item(ui, Command::ToggleSceneNumbers, numbers);
                self.checkbox_item(ui, Command::ToggleAutoContd, contd);
            });
            ui.menu_button(tr("Script"), |ui| {
                for c in [Command::EditTitlePage, Command::Reports, Command::MoveSceneUp, Command::MoveSceneDown, Command::AddSceneIds] {
                    self.menu_item(ui, c);
                }
            });
            ui.menu_button(tr("Production"), |ui| {
                self.menu_item(ui, Command::LockSceneNumbers);
                self.menu_item(ui, Command::UnlockSceneNumbers);
                ui.separator();
                let rev = self.doc.screenplay().revisions;
                let next = trf("Start Next Revision ({colour})", &[("colour", colour_label(rev.level.saturating_add(1)))]);
                if ui.button(next).clicked() {
                    let ctx = ui.ctx().clone();
                    self.run(&ctx, Command::StartNextRevision);
                    self.after_menu();
                    ui.close();
                }
                self.checkbox_item(ui, Command::ToggleRevisionTracking, rev.tracking);
                self.menu_item(ui, Command::ClearRevisionMarks);
            });
            ui.menu_button(tr("View"), |ui| {
                let (nav, preview, cards) = (self.show_navigator, self.view == View::Preview, self.view == View::Cards);
                self.checkbox_item(ui, Command::ToggleNavigator, nav);
                self.checkbox_item(ui, Command::TogglePagePreview, preview);
                self.checkbox_item(ui, Command::ToggleIndexCards, cards);
                ui.separator();
                let (focus, typewriter, dark) = (self.focus_mode, self.typewriter, self.dark_paper);
                self.checkbox_item(ui, Command::ToggleFocusMode, focus);
                self.checkbox_item(ui, Command::ToggleTypewriter, typewriter);
                self.checkbox_item(ui, Command::ToggleDarkPaper, dark);
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
            ui.add_enabled_ui(kind.is_some() && self.view == View::Draft, |ui| {
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
            for (view, label) in [(View::Draft, tr("Draft")), (View::Preview, tr("Page Preview")), (View::Cards, tr("Index Cards"))] {
                if ui.selectable_label(self.view == view, label).clicked() && self.view != view {
                    let ctx = ui.ctx().clone();
                    match view {
                        View::Draft => {
                            self.view = View::Draft;
                            self.editor.refocus();
                        }
                        View::Preview => self.view = View::Preview,
                        View::Cards => self.run(&ctx, Command::ToggleIndexCards),
                    }
                }
            }
            ui.separator();
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
        let revisions = self.doc.screenplay().revisions;
        let dash = "–".to_owned();
        ui.horizontal(|ui| {
            ui.label(trf("Page {page} of {pages}", &[("page", &page.map_or(dash.clone(), |p| p.to_string())), ("pages", &pages.to_string())]));
            ui.separator();
            ui.label(trf("Scene {scene} of {scenes}", &[("scene", &scene.map_or(dash.clone(), |s| s.to_string())), ("scenes", &scenes.to_string())]));
            ui.separator();
            // A live region, so a screen reader hears the element change after Tab or Ctrl+1…7.
            let kind_text = ui.label(kind.unwrap_or("–"));
            ui.ctx().accesskit_node_builder(kind_text.id, |node| node.set_live(egui::accesskit::Live::Polite));
            if revisions.level > 0 {
                ui.separator();
                let text = if revisions.tracking {
                    trf("{colour} revision, marking changes", &[("colour", colour_label(revisions.level))])
                } else {
                    trf("{colour} revision", &[("colour", colour_label(revisions.level))])
                };
                ui.label(text);
            }
            ui.separator();
            ui.label(if self.doc.is_dirty() { tr("Unsaved changes") } else { tr("Saved") });
            if !self.status.is_empty() {
                ui.separator();
                let status = ui.label(&self.status);
                ui.ctx().accesskit_node_builder(status.id, |node| node.set_live(egui::accesskit::Live::Polite));
            }
        });
    }

    fn dialogs(&mut self, ctx: &Context) {
        if let Some(path) = self.recovery.clone() {
            match dialogs::recover(ctx, &self.document_name()) {
                Some(true) => {
                    self.recovery = None;
                    match io::read_autosave(&path) {
                        Ok(sp) => {
                            self.doc.recover(sp);
                            self.editor = EditorState::new(self.doc.screenplay().elements.len());
                            self.editor.focus_at(0, 0);
                            self.set_status(tr("Recovered the unsaved changes; save to keep them"));
                        }
                        Err(e) => self.set_status(error_text(&e)),
                    }
                }
                Some(false) => {
                    self.recovery = None;
                    autosave::discard(self.doc.path.as_deref());
                }
                None => {}
            }
        }
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
        if self.reports_open {
            let sp = self.doc.screenplay().clone();
            let layout = self.doc.layout().clone();
            crate::reports::show(ctx, &mut self.reports_open, &sp, &layout);
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
pub(crate) trait WidgetInfoLabel {
    fn widget_info_label(self, name: &str) -> Self;
}

impl WidgetInfoLabel for egui::Response {
    fn widget_info_label(self, name: &str) -> Self {
        let name = name.to_owned();
        self.ctx.accesskit_node_builder(self.id, |node| node.set_label(name));
        self
    }
}
