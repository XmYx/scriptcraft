//! The draft editor: the script on one long sheet of paper, one text field per element, set at
//! the element's indent and width. Dashed rules mark where the printed pages break.

use egui::text::{CCursor, CCursorRange};
use egui::{Align, Align2, Color32, FontId, Id, Key, Margin, Rect, Stroke, TextEdit, Ui, pos2, vec2};
use scriptcraft_engine::{complete, typing};
use scriptcraft_layout::{Align as TextAlign, format_of};
use scriptcraft_model::ElementKind;

use crate::app::{ScriptApp, kind_label};
use crate::i18n::{tr, trf};
use crate::{fonts, keys};

/// Paper and ink colours. The page is always white with dark text, as printed, in both themes.
pub const PAPER: Color32 = Color32::from_rgb(0xFF, 0xFE, 0xFB);
pub const INK: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);
/// Notes, sections and other text that is not printed: 4.6:1 on the paper.
pub const MUTED: Color32 = Color32::from_rgb(0x6B, 0x6B, 0x6B);
/// Page-break rules: 3.6:1 on the paper (non-text contrast).
pub const RULE: Color32 = Color32::from_rgb(0x84, 0x84, 0x90);

/// Underline under misspelled words: 6.5:1 on the light paper, visible on the dark one.
pub const MISSPELLED: Color32 = Color32::from_rgb(0xC6, 0x28, 0x28);

/// The page's colours: paper, ink, muted text and rules. The dark page keeps the same contrast
/// rules (ink 13.9:1, muted 6.1:1, rules 3.5:1 on its paper).
#[derive(Clone, Copy)]
pub struct Palette {
    pub paper: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub rule: Color32,
}

pub const LIGHT: Palette = Palette { paper: PAPER, ink: INK, muted: MUTED, rule: RULE };
pub const DARK: Palette = Palette {
    paper: Color32::from_rgb(0x1E, 0x1F, 0x22),
    ink: Color32::from_rgb(0xE8, 0xE6, 0xE1),
    muted: Color32::from_rgb(0xA3, 0xA3, 0xA3),
    rule: Color32::from_rgb(0x7E, 0x7E, 0x8A),
};

/// Editor state that is not part of the document.
pub struct EditorState {
    /// A stable widget id per element, so focus and caret survive inserts above.
    ids: Vec<u64>,
    next_id: u64,
    /// The element the caret is in.
    pub focused: Option<usize>,
    /// Put the caret here on the next frame: element and character (`usize::MAX` = end).
    pending_focus: Option<(usize, usize)>,
    scroll_to: Option<usize>,
    suggestion: usize,
    /// Suggestions hidden with Escape for this element and text.
    dismissed: Option<(usize, String)>,
    /// Select this range on the next frame: element, start, length (a find match).
    pending_select: Option<(usize, usize, usize)>,
    /// The caret's character offset in the focused element, as of the last frame.
    pub caret_hint: Option<usize>,
    /// Misspelled byte ranges by text, so a line is checked once per change, not per frame.
    pub(crate) spell_cache: std::collections::HashMap<String, Vec<std::ops::Range<usize>>>,
}

impl EditorState {
    pub fn new(elements: usize) -> EditorState {
        let mut s = EditorState {
            ids: Vec::new(),
            next_id: 0,
            focused: None,
            pending_focus: None,
            scroll_to: None,
            suggestion: 0,
            dismissed: None,
            pending_select: None,
            spell_cache: std::collections::HashMap::new(),
            caret_hint: None,
        };
        s.reset_ids(elements);
        s
    }

    /// New ids for every element (after undo, redo or a replaced document).
    pub fn reset_ids(&mut self, elements: usize) {
        self.ids = (0..elements as u64).map(|i| self.next_id + i).collect();
        self.next_id += elements as u64;
    }

    fn insert_id(&mut self, at: usize) {
        let at = at.min(self.ids.len());
        self.ids.insert(at, self.next_id);
        self.next_id += 1;
    }

    fn remove_id(&mut self, at: usize) {
        if at < self.ids.len() {
            self.ids.remove(at);
        }
    }

    fn id(&mut self, i: usize) -> Id {
        while self.ids.len() <= i {
            self.insert_id(self.ids.len());
        }
        Id::unique(("scriptcraft-element", self.ids.get(i).copied().unwrap_or(u64::MAX)))
    }

    /// Put the caret in element `i` at character `char_index` (`usize::MAX` for the end) and
    /// scroll it into view.
    pub fn focus_at(&mut self, i: usize, char_index: usize) {
        self.pending_focus = Some((i, char_index));
        self.focused = Some(i);
        self.scroll_to = Some(i);
    }

    /// Focus the current element again (after undo or redo).
    pub fn refocus(&mut self) {
        self.focus_at(self.focused.unwrap_or(0), usize::MAX);
    }

    /// Scroll to and focus a scene (navigator).
    pub fn go_to(&mut self, i: usize) {
        self.focus_at(i, usize::MAX);
    }

    /// Put the caret on element `i` with characters `start..start + len` selected.
    pub fn select(&mut self, i: usize, start: usize, len: usize) {
        self.focus_at(i, start + len);
        self.pending_select = Some((i, start, len));
    }
}

/// What a key press asked for; applied after the elements are drawn.
enum Action {
    Enter { element: usize, at: usize, len: usize },
    Tab { element: usize, forward: bool },
    BackspaceAtStart(usize),
    FocusPrevious(usize),
    FocusNext(usize),
    Accept { element: usize, text: String, then_enter: bool },
}

struct Geometry {
    font: FontId,
    small: FontId,
    char_w: f32,
    row_h: f32,
    paper_left: f32,
    colours: Palette,
    typewriter: bool,
    spelling: bool,
}

pub fn show(app: &mut ScriptApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let font = fonts::script(12.0 * app.zoom);
    let char_w = ctx.fonts_mut(|f| f.glyph_width(&font, 'M')).max(1.0);
    let row_h = ctx.fonts_mut(|f| f.row_height(&font)).max(1.0);
    let ppi = char_w * 10.0;
    let (paper_w_in, paper_h_in) = app.doc.setup.paper.size_in();
    let paper_w = paper_w_in * ppi;
    let page_starts = app.doc.layout().page_starts.clone();
    let scene_labels = scene_labels(app);
    let n = app.doc.screenplay().elements.len();
    let mut rects: Vec<Option<Rect>> = vec![None; n];
    let mut action: Option<Action> = None;

    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("editor-scroll").show(ui, |ui| {
        ui.add_space(16.0);
        let side = ((ui.available_width() - paper_w) / 2.0).max(12.0);
        ui.horizontal_top(|ui| {
            ui.add_space(side);
            let colours = if app.dark_paper { DARK } else { LIGHT };
            egui::Frame::new().fill(colours.paper).stroke(Stroke::new(1.0, Color32::from_gray(150))).show(ui, |ui| {
                ui.set_width(paper_w);
                ui.set_min_height(paper_h_in * ppi);
                *ui.visuals_mut() = if app.dark_paper { egui::Visuals::dark() } else { egui::Visuals::light() };
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let geo = Geometry {
                    font: font.clone(),
                    small: fonts::script(9.0 * app.zoom),
                    char_w,
                    row_h,
                    paper_left: ui.min_rect().left(),
                    colours,
                    typewriter: app.typewriter,
                    spelling: app.spelling,
                };
                // The frame sits in a horizontal row; the page itself flows top to bottom.
                ui.vertical(|ui| {
                    ui.add_space(ppi);
                    for i in 0..n {
                        let label = scene_labels.get(i).cloned().flatten();
                        if let Some(slot) = rects.get_mut(i) {
                            *slot = element(app, ui, i, &geo, label, &mut action);
                        }
                    }
                    ui.add_space(ppi);
                    page_rules(ui, &geo, paper_w, &page_starts, &rects);
                });
            });
        });
        ui.add_space(32.0);
    });

    if let Some(action) = action {
        apply(app, action);
    }
}

/// Scene numbers for the left margin, by element.
fn scene_labels(app: &ScriptApp) -> Vec<Option<String>> {
    let mut n = 0;
    app.doc
        .screenplay()
        .elements
        .iter()
        .map(|e| {
            (e.kind == ElementKind::SceneHeading).then(|| {
                n += 1;
                e.scene_number.clone().unwrap_or_else(|| n.to_string())
            })
        })
        .collect()
}

fn element(app: &mut ScriptApp, ui: &mut Ui, i: usize, geo: &Geometry, scene_label: Option<String>, action: &mut Option<Action>) -> Option<Rect> {
    let e = app.doc.screenplay().elements.get(i)?.clone();
    let id = app.editor.id(i);
    let printed = format_of(e.kind);
    let (left, width, align, before) = match printed {
        Some(f) => (f.left, f.width, f.align, f.space_before),
        None => (15, 60, TextAlign::Left, 1),
    };
    if i > 0 {
        ui.add_space(before as f32 * geo.row_h);
    }

    // Caret placement asked for by the last edit or the navigator.
    if let Some((target, char_index)) = app.editor.pending_focus
        && target == i
    {
        let mut state = TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        let len = e.text.chars().count();
        let at = char_index.min(len);
        let range = match app.editor.pending_select.take() {
            Some((el, start, n)) if el == i => CCursorRange::two(CCursor::new(start.min(len)), CCursor::new((start + n).min(len))),
            _ => CCursorRange::one(CCursor::new(at)),
        };
        state.cursor.set_char_range(Some(range));
        TextEdit::store_state(ui.ctx(), id, state);
        ui.memory_mut(|m| m.request_focus(id));
        app.editor.pending_focus = None;
    }

    let focused = ui.memory(|m| m.has_focus(id));
    let len = e.text.chars().count();
    let suggestions = if focused && app.editor.dismissed.as_ref() != Some(&(i, e.text.clone())) {
        complete::suggestions(app.doc.screenplay(), i)
    } else {
        Vec::new()
    };
    if suggestions.is_empty() {
        app.editor.suggestion = 0;
    } else {
        app.editor.suggestion = app.editor.suggestion.min(suggestions.len() - 1);
    }
    if focused && action.is_none() {
        keyboard(app, ui, id, i, len, &e.text, &suggestions, action);
    }

    let mut buf = if e.kind.is_uppercase() { uppercase_same_length(&e.text) } else { e.text.clone() };
    let colour = if printed.is_some() { geo.colours.ink } else { geo.colours.muted };
    let egui_align = match align {
        TextAlign::Left => Align2::LEFT_TOP,
        TextAlign::Center => Align2::CENTER_TOP,
        TextAlign::Right => Align2::RIGHT_TOP,
    };
    let hint = kind_label(e.kind);
    if app.editor.spell_cache.len() > 4000 {
        app.editor.spell_cache.clear();
    }
    let check = geo.spelling && scriptcraft_engine::spelling::checked(e.kind);
    let cache = &mut app.editor.spell_cache;
    let font = geo.font.clone();
    let halign = egui_align.x();
    // Misspelled words get a red wavy-free underline (a plain 1.5 px rule, 6.5:1 on the paper).
    let mut layouter = move |ui: &Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
        let text = buf.as_str();
        let ranges =
            if check { cache.entry(text.to_owned()).or_insert_with(|| scriptcraft_engine::spelling::misspelled(text)).clone() } else { Vec::new() };
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = wrap_width;
        job.halign = halign;
        let plain = egui::TextFormat { font_id: font.clone(), color: colour, ..Default::default() };
        let marked = egui::TextFormat { underline: Stroke::new(1.5, MISSPELLED), ..plain.clone() };
        let mut at = 0;
        for r in &ranges {
            if r.start < at || r.end > text.len() {
                continue;
            }
            job.append(text.get(at..r.start).unwrap_or(""), 0.0, plain.clone());
            job.append(text.get(r.clone()).unwrap_or(""), 0.0, marked.clone());
            at = r.end;
        }
        job.append(text.get(at..).unwrap_or(""), 0.0, plain);
        ui.fonts_mut(|f| f.layout_job(job))
    };
    let output = ui
        .horizontal_top(|ui| {
            ui.add_space(left as f32 * geo.char_w);
            TextEdit::multiline(&mut buf)
                .id(id)
                .font(geo.font.clone())
                .text_color(colour)
                .frame(egui::Frame::NONE)
                .margin(Margin::ZERO)
                .desired_width(width as f32 * geo.char_w + geo.char_w)
                .desired_rows(1)
                .lock_focus(true)
                .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, Key::Enter)))
                .align(egui_align)
                .hint_text(hint)
                .layouter(&mut layouter)
                .show(ui)
        })
        .inner;
    let response = output.response.response;
    // The field's accessible name: its element kind, and the scene number on a heading.
    let name = match &scene_label {
        Some(n) => trf("{kind} {number}", &[("kind", kind_label(e.kind)), ("number", n)]),
        None => kind_label(e.kind).to_owned(),
    };
    ui.ctx().accesskit_node_builder(response.id, |node| node.set_label(name));
    if response.has_focus() {
        app.editor.focused = Some(i);
        app.editor.caret_hint = output.cursor_range.map(|r| r.primary.index.0);
    }
    if response.changed() && e.kind != ElementKind::PageBreak {
        let text = if e.kind.is_uppercase() { uppercase_same_length(&buf) } else { buf };
        app.doc.set_text(i, text);
        if geo.typewriter {
            // Typewriter scrolling: the line being typed stays in the middle of the window.
            response.scroll_to_me(Some(Align::Center));
        }
    }
    if app.editor.scroll_to == Some(i) {
        response.scroll_to_me(Some(Align::Center));
        app.editor.scroll_to = None;
    }
    let rect = response.rect;
    let painter = ui.painter();
    if let Some(label) = scene_label {
        let x = geo.paper_left + 10.0 * geo.char_w;
        painter.text(pos2(x, rect.top()), Align2::RIGHT_TOP, label, geo.font.clone(), geo.colours.muted);
    }
    if e.revision > 0 {
        // The revision mark in the right margin, as printed.
        painter.text(pos2(geo.paper_left + 80.0 * geo.char_w, rect.top()), Align2::LEFT_TOP, "*", geo.font.clone(), geo.colours.ink);
    }
    if printed.is_none() {
        let tag = match e.kind {
            ElementKind::Section => tr("section"),
            ElementKind::Synopsis => tr("synopsis"),
            ElementKind::Note => tr("note"),
            ElementKind::Boneyard => tr("omitted"),
            _ => "",
        };
        painter.text(pos2(geo.paper_left + 2.0 * geo.char_w, rect.top() + 2.0), Align2::LEFT_TOP, tag, geo.small.clone(), geo.colours.muted);
    }
    if focused && !suggestions.is_empty() {
        suggestion_popup(app, ui, rect, i, &suggestions, geo, action);
    }
    Some(rect)
}

/// Capitals for display, unless upper-casing would change the length (`ß` → `SS`) and move the
/// caret.
fn uppercase_same_length(text: &str) -> String {
    let up = text.to_uppercase();
    if up.chars().count() == text.chars().count() { up } else { text.to_owned() }
}

#[allow(clippy::too_many_arguments)]
fn keyboard(app: &mut ScriptApp, ui: &Ui, id: Id, i: usize, len: usize, text: &str, suggestions: &[String], action: &mut Option<Action>) {
    let range = TextEdit::load_state(ui.ctx(), id).and_then(|s| s.cursor.char_range());
    let at = range.map_or(len, |r| r.primary.index.0).min(len);
    let collapsed = range.is_none_or(|r| r.primary.index.0 == r.secondary.index.0);
    let count = app.doc.screenplay().elements.len();
    if !suggestions.is_empty() {
        if keys::take(ui, Key::ArrowDown, false) {
            app.editor.suggestion = (app.editor.suggestion + 1) % suggestions.len();
        }
        if keys::take(ui, Key::ArrowUp, false) {
            app.editor.suggestion = (app.editor.suggestion + suggestions.len() - 1) % suggestions.len();
        }
        if keys::take(ui, Key::Escape, false) {
            app.editor.dismissed = Some((i, text.to_owned()));
            return;
        }
        let enter = keys::take(ui, Key::Enter, false);
        if enter || keys::take(ui, Key::Tab, false) {
            if let Some(choice) = suggestions.get(app.editor.suggestion) {
                let kind = app.doc.screenplay().elements.get(i).map(|e| e.kind);
                *action = Some(Action::Accept { element: i, text: choice.clone(), then_enter: enter && kind == Some(ElementKind::Character) });
            }
            return;
        }
    }
    *action = if keys::take(ui, Key::Enter, false) {
        Some(Action::Enter { element: i, at, len })
    } else if keys::take(ui, Key::Tab, false) {
        Some(Action::Tab { element: i, forward: true })
    } else if keys::take(ui, Key::Tab, true) {
        Some(Action::Tab { element: i, forward: false })
    } else if collapsed && at == 0 && i > 0 && keys::peek(ui, Key::Backspace, false) {
        keys::take(ui, Key::Backspace, false);
        Some(Action::BackspaceAtStart(i))
    } else if collapsed && at == 0 && i > 0 && keys::peek(ui, Key::ArrowUp, false) {
        keys::take(ui, Key::ArrowUp, false);
        Some(Action::FocusPrevious(i))
    } else if collapsed && at == len && i + 1 < count && keys::peek(ui, Key::ArrowDown, false) {
        keys::take(ui, Key::ArrowDown, false);
        Some(Action::FocusNext(i))
    } else {
        None
    };
}

fn suggestion_popup(app: &mut ScriptApp, ui: &Ui, rect: Rect, i: usize, suggestions: &[String], geo: &Geometry, action: &mut Option<Action>) {
    egui::Area::new(Id::unique("scriptcraft-suggestions")).order(egui::Order::Foreground).fixed_pos(rect.left_bottom() + vec2(0.0, 2.0)).show(
        ui.ctx(),
        |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(tr("Enter or Tab to accept · Esc to hide")).small().weak());
                for (k, s) in suggestions.iter().enumerate() {
                    let selected = k == app.editor.suggestion;
                    let label = egui::RichText::new(s).font(FontId::new(geo.font.size * 0.9, geo.font.family.clone()));
                    if ui.add(egui::Button::selectable(selected, label)).clicked() {
                        *action = Some(Action::Accept { element: i, text: s.clone(), then_enter: false });
                    }
                }
            });
        },
    );
}

/// Dashed rules where printed pages start, with the page number at the right margin.
fn page_rules(ui: &Ui, geo: &Geometry, paper_w: f32, starts: &[scriptcraft_layout::PageStart], rects: &[Option<Rect>]) {
    let painter = ui.painter();
    let (left, right) = (geo.paper_left, geo.paper_left + paper_w);
    for start in starts.iter().filter(|s| s.page > 1) {
        let Some(Some(rect)) = rects.get(start.element) else { continue };
        let y = if start.line_in_element == 0 { rect.top() - geo.row_h * 0.5 } else { rect.top() + start.line_in_element as f32 * geo.row_h };
        painter.extend(egui::Shape::dashed_line(&[pos2(left + 4.0, y), pos2(right - 4.0, y)], Stroke::new(1.0, geo.colours.rule), 6.0, 5.0));
        painter.text(pos2(right - 8.0, y - 2.0), Align2::RIGHT_BOTTOM, format!("{}.", start.page), geo.small.clone(), geo.colours.muted);
    }
}

fn apply(app: &mut ScriptApp, action: Action) {
    match action {
        Action::Enter { element, at, len } => {
            let opened_above = at == 0 && len > 0;
            if let Some(caret) = app.doc.edit(|sp| typing::enter(sp, element, at)) {
                app.editor.insert_id(if opened_above { element } else { element + 1 });
                app.editor.focus_at(caret.element, caret.char_index);
            }
        }
        Action::Tab { element, forward } => {
            let Some(kind) = app.doc.screenplay().elements.get(element).map(|e| e.kind) else { return };
            let next = typing::on_tab(kind, forward);
            if let Some(caret) = app.doc.edit(|sp| typing::set_kind(sp, element, next)) {
                app.editor.focus_at(caret.element, caret.char_index);
            }
        }
        Action::BackspaceAtStart(element) => {
            if let Some(caret) = app.doc.edit(|sp| typing::backspace_at_start(sp, element)) {
                app.editor.remove_id(element);
                app.editor.focus_at(caret.element, caret.char_index);
            }
        }
        Action::FocusPrevious(element) => app.editor.focus_at(element.saturating_sub(1), usize::MAX),
        Action::FocusNext(element) => app.editor.focus_at(element + 1, 0),
        Action::Accept { element, text, then_enter } => {
            let len = text.chars().count();
            app.doc.set_text(element, text);
            if then_enter {
                apply(app, Action::Enter { element, at: len, len });
            } else {
                app.editor.focus_at(element, usize::MAX);
            }
        }
    }
    if app.editor.ids.len() != app.doc.screenplay().elements.len() {
        app.editor.reset_ids(app.doc.screenplay().elements.len());
    }
}
