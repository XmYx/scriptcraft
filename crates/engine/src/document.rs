//! The open document.

use std::path::{Path, PathBuf};

use scriptcraft_layout::{Layout, PageSetup, paginate};
use scriptcraft_model::{Element, ElementKind, Screenplay};

use crate::io::{self, EngineError};

/// Undo steps kept.
const UNDO_LIMIT: usize = 200;

/// The open script.
pub struct Document {
    screenplay: Screenplay,
    /// Where it is saved as Fountain; `None` until the first save (or after opening FDX).
    pub path: Option<PathBuf>,
    pub setup: PageSetup,
    dirty: bool,
    undo: Vec<Screenplay>,
    redo: Vec<Screenplay>,
    /// The element being typed into since the last undo step, so typing in one element is one step.
    typing_in: Option<usize>,
    generation: u64,
    layout: Option<(u64, PageSetup, Layout)>,
}

impl Default for Document {
    fn default() -> Self {
        Document::new()
    }
}

impl Document {
    /// A new script: one empty scene heading.
    pub fn new() -> Document {
        let sp = Screenplay { elements: vec![Element::new(ElementKind::SceneHeading, "")], ..Screenplay::default() };
        Document::from_screenplay(sp, None)
    }

    pub fn from_screenplay(mut screenplay: Screenplay, path: Option<PathBuf>) -> Document {
        if screenplay.elements.is_empty() {
            screenplay.elements.push(Element::new(ElementKind::SceneHeading, ""));
        }
        Document {
            screenplay,
            path,
            setup: PageSetup::default(),
            dirty: false,
            undo: Vec::new(),
            redo: Vec::new(),
            typing_in: None,
            generation: 0,
            layout: None,
        }
    }

    /// Open a Fountain or FDX file. FDX opens as an untitled Fountain script (saving asks for a
    /// name). Also returns the file kind and, for FDX, the paragraph types read as action.
    pub fn open(path: &Path) -> Result<(Document, io::FileKind, Vec<String>), EngineError> {
        let opened = io::open(path)?;
        let save_path = (opened.kind == io::FileKind::Fountain).then(|| path.to_path_buf());
        Ok((Document::from_screenplay(opened.screenplay, save_path), opened.kind, opened.unknown_types))
    }

    pub fn screenplay(&self) -> &Screenplay {
        &self.screenplay
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// The name for the window title: the file name, else the script's title; `None` for an
    /// untitled script (the interface shows its own word for that).
    pub fn name(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .or_else(|| self.screenplay.title().map(str::to_owned))
    }

    /// Change the script as one undo step.
    pub fn edit<R>(&mut self, change: impl FnOnce(&mut Screenplay) -> R) -> R {
        self.push_undo();
        self.typing_in = None;
        let r = change(&mut self.screenplay);
        self.changed();
        r
    }

    /// Replace one element's text. Consecutive calls for the same element are one undo step.
    pub fn set_text(&mut self, index: usize, text: String) {
        if self.screenplay.elements.get(index).is_none_or(|e| e.text == text) {
            return;
        }
        if self.typing_in != Some(index) {
            self.push_undo();
            self.typing_in = Some(index);
        }
        if let Some(e) = self.screenplay.elements.get_mut(index) {
            e.text = text;
        }
        self.changed();
    }

    /// Replace the whole script (title page dialog, imports) as one undo step.
    pub fn replace(&mut self, screenplay: Screenplay) {
        self.edit(|sp| *sp = screenplay);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(std::mem::replace(&mut self.screenplay, previous));
        self.typing_in = None;
        self.changed();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(std::mem::replace(&mut self.screenplay, next));
        self.typing_in = None;
        self.changed();
        true
    }

    /// Save as Fountain to the current path.
    pub fn save(&mut self) -> Result<(), EngineError> {
        let Some(path) = self.path.clone() else { return Err(EngineError::NoPath) };
        self.save_as(&path)
    }

    /// Save as Fountain to `path` and make it the current path.
    pub fn save_as(&mut self, path: &Path) -> Result<(), EngineError> {
        io::save_fountain(path, &self.screenplay)?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        self.typing_in = None;
        Ok(())
    }

    /// The layout for the current script and page setup, recomputed only after a change.
    pub fn layout(&mut self) -> &Layout {
        let fresh = matches!(&self.layout, Some((g, s, _)) if *g == self.generation && *s == self.setup);
        if !fresh {
            self.layout = None;
        }
        let (generation, setup, screenplay) = (self.generation, self.setup, &self.screenplay);
        &self.layout.get_or_insert_with(|| (generation, setup, paginate(screenplay, &setup))).2
    }

    fn push_undo(&mut self) {
        self.undo.push(self.screenplay.clone());
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn changed(&mut self) {
        self.dirty = true;
        self.generation = self.generation.wrapping_add(1);
        if self.screenplay.elements.is_empty() {
            self.screenplay.elements.push(Element::new(ElementKind::Action, ""));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_in_one_element_is_one_undo_step() {
        let mut doc = Document::new();
        doc.set_text(0, "I".into());
        doc.set_text(0, "IN".into());
        doc.set_text(0, "INT".into());
        doc.edit(|sp| sp.elements.push(Element::new(ElementKind::Action, "x")));
        doc.set_text(1, "xy".into());
        assert!(doc.undo());
        assert_eq!(doc.screenplay().elements.len(), 2);
        assert!(doc.undo());
        assert_eq!(doc.screenplay().elements.len(), 1);
        assert!(doc.undo());
        assert_eq!(doc.screenplay().elements[0].text, "");
        assert!(!doc.undo());
        assert!(doc.redo());
        assert_eq!(doc.screenplay().elements[0].text, "INT");
    }

    #[test]
    fn layout_is_cached_until_a_change() {
        let mut doc = Document::new();
        let pages = doc.layout().page_count();
        doc.set_text(0, "INT. A - DAY".into());
        assert_eq!(doc.layout().page_count(), pages);
        assert_eq!(doc.layout().pages[0].lines[0].text, "INT. A - DAY");
    }

    #[test]
    fn a_document_never_becomes_empty() {
        let mut doc = Document::new();
        doc.edit(|sp| sp.elements.clear());
        assert_eq!(doc.screenplay().elements.len(), 1);
    }
}
