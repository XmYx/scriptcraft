//! Autosave and backups, so a crash or a bad save never costs more than a minute of work.
//!
//! - **Autosave:** while a script has unsaved changes it is written every
//!   [`AUTOSAVE_SECONDS`] to `.<name>.autosave` beside the file (untitled scripts: the system's
//!   temporary folder). A successful save removes it. When ScriptCraft opens a file whose
//!   autosave is newer, it offers to recover it.
//! - **Backup:** each save first copies the previous version to `.<name>.backup`.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Seconds between autosaves of a changed script.
pub const AUTOSAVE_SECONDS: u64 = 60;

/// Where the autosave of `path` (or of an untitled script) goes.
pub fn autosave_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(p) => sibling(p, "autosave"),
        None => std::env::temp_dir().join("scriptcraft-untitled.autosave"),
    }
}

/// Where the backup of `path` goes.
pub fn backup_path(path: &Path) -> PathBuf {
    sibling(path, "backup")
}

fn sibling(path: &Path, kind: &str) -> PathBuf {
    let name = path.file_name().map_or_else(|| "script".into(), |n| n.to_string_lossy().into_owned());
    path.with_file_name(format!(".{name}.{kind}"))
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// An autosave of `path` that is newer than the file itself, if one exists.
pub fn recovery_for(path: Option<&Path>) -> Option<PathBuf> {
    let autosave = autosave_path(path);
    let saved = modified(&autosave)?;
    match path.and_then(modified) {
        Some(file) if file >= saved => None,
        _ => Some(autosave),
    }
}

/// Remove the autosave of `path` (after a save, or when the user declines recovery).
pub fn discard(path: Option<&Path>) {
    let _ = std::fs::remove_file(autosave_path(path));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_recovery() {
        let dir = std::env::temp_dir().join(format!("scriptcraft-autosave-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("a.fountain");
        std::fs::write(&file, "INT. A - DAY\n").unwrap();
        assert_eq!(autosave_path(Some(&file)), dir.join(".a.fountain.autosave"));
        assert_eq!(backup_path(&file), dir.join(".a.fountain.backup"));
        assert_eq!(recovery_for(Some(&file)), None);
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(autosave_path(Some(&file)), "INT. B - DAY\n").unwrap();
        assert_eq!(recovery_for(Some(&file)), Some(autosave_path(Some(&file))));
        discard(Some(&file));
        assert_eq!(recovery_for(Some(&file)), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
