//! Files: opening Fountain and FDX, saving Fountain without risking the old copy, exports.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use scriptcraft_layout::{PageSetup, paginate, render_text};
use scriptcraft_model::Screenplay;

/// Why a file action failed. Every variant names what to do next in its message.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("could not read {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("could not write {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error("{path} is {size} bytes; files over 64 MiB are not opened")]
    TooLarge { path: PathBuf, size: u64 },
    #[error("{0}")]
    Fountain(#[from] scriptcraft_fountain::Error),
    #[error("{path}: {source}")]
    Fdx { path: PathBuf, source: scriptcraft_fdx::Error },
    #[error("could not write the scene list: {0}")]
    SceneList(String),
    #[error("this script has not been saved yet; choose Save As")]
    NoPath,
    #[error("could not write the PDF: {0}")]
    Pdf(String),
}

/// The kinds of file ScriptCraft opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Fountain,
    Fdx,
}

impl FileKind {
    /// From the extension: `.fdx` is FDX, anything else is read as Fountain (`.fountain`, `.spmd`,
    /// `.txt`).
    pub fn of(path: &Path) -> FileKind {
        match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("fdx") => FileKind::Fdx,
            _ => FileKind::Fountain,
        }
    }
}

/// An opened file.
pub struct Opened {
    pub screenplay: Screenplay,
    pub kind: FileKind,
    /// FDX paragraph types read as action because ScriptCraft has no such element.
    pub unknown_types: Vec<String>,
}

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn read_bounded(path: &Path) -> Result<Vec<u8>, EngineError> {
    let size = std::fs::metadata(path).map_err(|source| EngineError::Read { path: path.to_path_buf(), source })?.len();
    if size > MAX_FILE_BYTES {
        return Err(EngineError::TooLarge { path: path.to_path_buf(), size });
    }
    std::fs::read(path).map_err(|source| EngineError::Read { path: path.to_path_buf(), source })
}

/// Open a Fountain or FDX file.
pub fn open(path: &Path) -> Result<Opened, EngineError> {
    let bytes = read_bounded(path)?;
    match FileKind::of(path) {
        FileKind::Fountain => {
            Ok(Opened { screenplay: scriptcraft_fountain::parse_bytes(&bytes)?, kind: FileKind::Fountain, unknown_types: Vec::new() })
        }
        FileKind::Fdx => {
            let (screenplay, report) = scriptcraft_fdx::import(&bytes).map_err(|source| EngineError::Fdx { path: path.to_path_buf(), source })?;
            Ok(Opened { screenplay, kind: FileKind::Fdx, unknown_types: report.unknown_types })
        }
    }
}

/// Write `bytes` to `path` through a temporary file in the same folder, renamed over the old
/// file only once fully written, so a failed save never leaves a half-written script.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let err = |source| EngineError::Write { path: path.to_path_buf(), source };
    let name = path.file_name().map_or_else(|| "script".into(), |n| n.to_string_lossy().into_owned());
    let tmp = path.with_file_name(format!(".{name}.scriptcraft-tmp"));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result.map_err(err)
}

/// Fountain text for a script, with its revision marks kept (see [`crate::revisions`]).
pub fn fountain_text(sp: &Screenplay) -> String {
    match crate::revisions::with_block(sp) {
        Some(with) => scriptcraft_fountain::write(&with),
        None => scriptcraft_fountain::write(sp),
    }
}

/// Save as Fountain. The previous version is kept as a backup ([`crate::autosave::backup_path`])
/// and the autosave is removed.
pub fn save_fountain(path: &Path, sp: &Screenplay) -> Result<(), EngineError> {
    if path.is_file() {
        let _ = std::fs::copy(path, crate::autosave::backup_path(path));
    }
    write_atomic(path, fountain_text(sp).as_bytes())?;
    crate::autosave::discard(Some(path));
    Ok(())
}

/// Write the autosave of a script (see [`crate::autosave`]).
pub fn autosave(path: Option<&Path>, sp: &Screenplay) -> Result<(), EngineError> {
    write_atomic(&crate::autosave::autosave_path(path), fountain_text(sp).as_bytes())
}

/// Read an autosave back.
pub fn read_autosave(autosave: &Path) -> Result<Screenplay, EngineError> {
    let bytes = read_bounded(autosave)?;
    let mut sp = scriptcraft_fountain::parse_bytes(&bytes)?;
    crate::revisions::take(&mut sp);
    Ok(sp)
}

/// Save as FDX (an export: the script keeps its Fountain file). Returns how many outline and note
/// elements FDX could not carry.
pub fn save_fdx(path: &Path, sp: &Screenplay) -> Result<usize, EngineError> {
    let (xml, report) = scriptcraft_fdx::export(sp);
    write_atomic(path, xml.as_bytes())?;
    Ok(report.left_out)
}

/// The scene list for storyboards as JSON (see `docs/interop.md`).
pub fn scene_list_json(sp: &Screenplay, setup: &PageSetup, source: Option<String>) -> Result<String, EngineError> {
    let layout = paginate(sp, setup);
    let list = scriptcraft_interchange::scene_list(sp, &layout, source);
    scriptcraft_interchange::to_json(&list).map_err(|e| EngineError::SceneList(e.to_string()))
}

/// The paginated script as PDF, in Liberation Mono (or the fallback face), with revision labels
/// in English. The interface passes its own labels through [`pdf_with`].
pub fn pdf(sp: &Screenplay, setup: &PageSetup) -> Result<Vec<u8>, EngineError> {
    pdf_with(sp, setup, |level| format!("{} Revision", scriptcraft_model::revision_colour(level)))
}

/// [`pdf`] with the revision label for each level supplied by the caller.
pub fn pdf_with(sp: &Screenplay, setup: &PageSetup, revision_label: fn(u8) -> String) -> Result<Vec<u8>, EngineError> {
    let layout = paginate(sp, setup);
    let (_, font) = scriptcraft_fonts::pdf_face();
    let author = sp.title_page.get("Author").map(str::to_owned);
    let info = scriptcraft_pdf::Info { title: sp.title().map(str::to_owned), author, revision_label: Some(revision_label) };
    scriptcraft_pdf::export(&layout, font, &info).map_err(|e| EngineError::Pdf(e.to_string()))
}

/// The paginated script as plain text, pages separated by form feeds.
pub fn plain_text(sp: &Screenplay, setup: &PageSetup) -> String {
    render_text(&paginate(sp, setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("scriptcraft-engine-{}-{name}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn save_and_open_fountain() {
        let dir = scratch("save");
        let path = dir.join("a.fountain");
        let sp = scriptcraft_fountain::parse("INT. A - DAY\n\nShe waits.\n");
        save_fountain(&path, &sp).unwrap();
        let opened = open(&path).unwrap();
        assert_eq!(opened.screenplay, sp);
        assert_eq!(opened.kind, FileKind::Fountain);
        assert!(std::fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().contains("tmp")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(matches!(open(Path::new("/nonexistent/x.fountain")), Err(EngineError::Read { .. })));
    }

    #[test]
    fn kind_from_extension() {
        assert_eq!(FileKind::of(Path::new("a.FDX")), FileKind::Fdx);
        assert_eq!(FileKind::of(Path::new("a.fountain")), FileKind::Fountain);
        assert_eq!(FileKind::of(Path::new("noext")), FileKind::Fountain);
    }
}
