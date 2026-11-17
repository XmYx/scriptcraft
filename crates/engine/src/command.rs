//! Every action, by id. The menu bar, keyboard shortcuts, the command-line tool and agents all
//! name actions this way; labels are what users see.

use scriptcraft_model::ElementKind;

/// A ScriptCraft action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    New,
    Open,
    OpenSample,
    Save,
    SaveAs,
    ExportSceneList,
    ExportText,
    ExportPdf,
    ExportFdx,
    Quit,
    Undo,
    Redo,
    Find,
    Replace,
    FindNext,
    FindPrevious,
    ToggleSpelling,
    NextMisspelling,
    AddToDictionary,
    /// Make the current element this kind.
    SetKind(ElementKind),
    EditTitlePage,
    AddSceneIds,
    Reports,
    MoveSceneUp,
    MoveSceneDown,
    LockSceneNumbers,
    UnlockSceneNumbers,
    StartNextRevision,
    ToggleRevisionTracking,
    ClearRevisionMarks,
    ToggleAutoContd,
    PaperUsLetter,
    PaperA4,
    ToggleSceneNumbers,
    ToggleNavigator,
    TogglePagePreview,
    ToggleIndexCards,
    ToggleFocusMode,
    ToggleTypewriter,
    ToggleDarkPaper,
    ZoomIn,
    ZoomOut,
    KeyboardHelp,
    About,
}

impl Command {
    /// Every command, in menu order.
    pub fn all() -> Vec<Command> {
        let mut all = vec![
            Command::New,
            Command::Open,
            Command::OpenSample,
            Command::Save,
            Command::SaveAs,
            Command::ExportSceneList,
            Command::ExportText,
            Command::ExportPdf,
            Command::ExportFdx,
            Command::Quit,
            Command::Undo,
            Command::Redo,
            Command::Find,
            Command::Replace,
            Command::FindNext,
            Command::FindPrevious,
            Command::ToggleSpelling,
            Command::NextMisspelling,
            Command::AddToDictionary,
        ];
        all.extend(ElementKind::WRITING.into_iter().map(Command::SetKind));
        all.extend([
            Command::EditTitlePage,
            Command::AddSceneIds,
            Command::Reports,
            Command::MoveSceneUp,
            Command::MoveSceneDown,
            Command::LockSceneNumbers,
            Command::UnlockSceneNumbers,
            Command::StartNextRevision,
            Command::ToggleRevisionTracking,
            Command::ClearRevisionMarks,
            Command::ToggleAutoContd,
            Command::PaperUsLetter,
            Command::PaperA4,
            Command::ToggleSceneNumbers,
            Command::ToggleNavigator,
            Command::TogglePagePreview,
            Command::ToggleIndexCards,
            Command::ToggleFocusMode,
            Command::ToggleTypewriter,
            Command::ToggleDarkPaper,
            Command::ZoomIn,
            Command::ZoomOut,
            Command::KeyboardHelp,
            Command::About,
        ]);
        all
    }

    /// The stable id: `file.save`, `element.scene_heading`.
    pub fn id(self) -> String {
        let fixed = match self {
            Command::New => "file.new",
            Command::Open => "file.open",
            Command::OpenSample => "file.open_sample",
            Command::Save => "file.save",
            Command::SaveAs => "file.save_as",
            Command::ExportSceneList => "file.export_scene_list",
            Command::ExportText => "file.export_text",
            Command::ExportPdf => "file.export_pdf",
            Command::ExportFdx => "file.export_fdx",
            Command::Quit => "file.quit",
            Command::Undo => "edit.undo",
            Command::Redo => "edit.redo",
            Command::Find => "edit.find",
            Command::Replace => "edit.replace",
            Command::FindNext => "edit.find_next",
            Command::FindPrevious => "edit.find_previous",
            Command::ToggleSpelling => "edit.check_spelling",
            Command::NextMisspelling => "edit.next_misspelling",
            Command::AddToDictionary => "edit.add_to_dictionary",
            Command::SetKind(kind) => return format!("element.{}", kind.id()),
            Command::EditTitlePage => "script.title_page",
            Command::AddSceneIds => "script.add_scene_ids",
            Command::Reports => "script.reports",
            Command::MoveSceneUp => "script.move_scene_up",
            Command::MoveSceneDown => "script.move_scene_down",
            Command::LockSceneNumbers => "production.lock_scene_numbers",
            Command::UnlockSceneNumbers => "production.unlock_scene_numbers",
            Command::StartNextRevision => "production.next_revision",
            Command::ToggleRevisionTracking => "production.track_revisions",
            Command::ClearRevisionMarks => "production.clear_revision_marks",
            Command::ToggleAutoContd => "format.auto_contd",
            Command::PaperUsLetter => "format.paper.us_letter",
            Command::PaperA4 => "format.paper.a4",
            Command::ToggleSceneNumbers => "format.scene_numbers",
            Command::ToggleNavigator => "view.navigator",
            Command::TogglePagePreview => "view.page_preview",
            Command::ToggleIndexCards => "view.index_cards",
            Command::ToggleFocusMode => "view.focus_mode",
            Command::ToggleTypewriter => "view.typewriter",
            Command::ToggleDarkPaper => "view.dark_paper",
            Command::ZoomIn => "view.zoom_in",
            Command::ZoomOut => "view.zoom_out",
            Command::KeyboardHelp => "help.keyboard",
            Command::About => "help.about",
        };
        fixed.to_owned()
    }

    pub fn from_id(id: &str) -> Option<Command> {
        Command::all().into_iter().find(|c| c.id() == id)
    }

    /// The menu label in English: the source text the interface translates.
    pub fn label(self) -> &'static str {
        match self {
            Command::New => "New",
            Command::Open => "Open…",
            Command::OpenSample => "Open Sample Script",
            Command::Save => "Save",
            Command::SaveAs => "Save As…",
            Command::ExportSceneList => "Export Scene List for Storyboards…",
            Command::ExportText => "Export as Plain Text…",
            Command::ExportPdf => "Export PDF…",
            Command::ExportFdx => "Export FDX…",
            Command::Quit => "Quit",
            Command::Undo => "Undo",
            Command::Redo => "Redo",
            Command::Find => "Find…",
            Command::Replace => "Find and Replace…",
            Command::FindNext => "Find Next",
            Command::FindPrevious => "Find Previous",
            Command::ToggleSpelling => "Check Spelling as You Type",
            Command::NextMisspelling => "Next Misspelling",
            Command::AddToDictionary => "Add Word to Dictionary",
            Command::SetKind(kind) => kind.label(),
            Command::EditTitlePage => "Title Page…",
            Command::AddSceneIds => "Add Stable Scene IDs",
            Command::Reports => "Reports…",
            Command::MoveSceneUp => "Move Scene Earlier",
            Command::MoveSceneDown => "Move Scene Later",
            Command::LockSceneNumbers => "Lock Scene Numbers",
            Command::UnlockSceneNumbers => "Unlock Scene Numbers",
            Command::StartNextRevision => "Start Next Revision",
            Command::ToggleRevisionTracking => "Mark Revisions",
            Command::ClearRevisionMarks => "Clear Revision Marks",
            Command::ToggleAutoContd => "Automatic (CONT'D)",
            Command::PaperUsLetter => "US Letter",
            Command::PaperA4 => "A4",
            Command::ToggleSceneNumbers => "Scene Numbers",
            Command::ToggleNavigator => "Scene Navigator",
            Command::TogglePagePreview => "Page Preview",
            Command::ToggleIndexCards => "Index Cards",
            Command::ToggleFocusMode => "Focus Mode",
            Command::ToggleTypewriter => "Typewriter Scrolling",
            Command::ToggleDarkPaper => "Dark Page",
            Command::ZoomIn => "Zoom In",
            Command::ZoomOut => "Zoom Out",
            Command::KeyboardHelp => "Keyboard Shortcuts",
            Command::About => "About ScriptCraft",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_round_trip() {
        let all = Command::all();
        for c in &all {
            assert_eq!(Command::from_id(&c.id()), Some(*c));
        }
        let mut ids: Vec<String> = all.iter().map(|c| c.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
    }
}
