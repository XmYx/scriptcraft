//! `cargo xtask <task>`: repository checks.
//!
//! - `ci`: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, then `layers` and `assets`.
//! - `layers`: crates below the UI must not depend on egui, eframe, winit or rfd (AGENTS.md §3).
//! - `assets`: every file under `assets/` and `docs/images/` is named in ATTRIBUTION.md (AGENTS.md §5).
//! - `licenses`: every dependency licence is on the Craft allowlist.
//! - `strings [--check | --shared <craft-i18n>]`: the interface source texts (`i18n/source-strings.tsv`).
//! - `progress`: progress-log and release-note fragments are well formed (RFC-0017).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod icons;
mod licenses;
mod strings;

use std::path::{Path, PathBuf};
use std::process::Command;

/// Crates that may use the UI toolkit.
const UI_CRATES: [&str; 2] = ["scriptcraft-ui-egui", "scriptcraft"];
const UI_DEPS: [&str; 4] = ["egui", "eframe", "winit", "rfd"];

fn main() {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "layers" => layers(),
        "assets" => assets(),
        "licenses" => licenses::run(&root()),
        "strings" => strings::run(&root(), &std::env::args().skip(2).collect::<Vec<_>>()),
        "progress" => progress(),
        "icons" => icons::run(&root()),
        _ => Err("usage: cargo xtask <ci | layers | assets | licenses | strings | progress | icons>".to_owned()),
    };
    if let Err(e) = result {
        eprintln!("xtask {task}: {e}");
        std::process::exit(1);
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn cargo(args: &[&str]) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    println!("$ cargo {}", args.join(" "));
    let status = Command::new(cargo).args(args).current_dir(root()).status().map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("cargo {} failed", args.join(" "))) }
}

fn ci() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo(&["test", "--workspace"])?;
    layers()?;
    assets()?;
    licenses::run(&root())?;
    strings::run(&root(), &["--check".to_owned()])?;
    progress()
}

/// RFC-0017 fragments: `docs/progress.d/<issue>-<branch-slug>.md` and
/// `packaging/release-notes.d/<issue>-<branch-slug>.md`, each one non-empty bullet entry.
fn progress() -> Result<(), String> {
    let mut problems = Vec::new();
    let mut count = 0;
    for dir in ["docs/progress.d", "packaging/release-notes.d"] {
        let path = root().join(dir);
        let Ok(entries) = std::fs::read_dir(&path) else { continue };
        for entry in entries.flatten() {
            let file = entry.path();
            let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if name == "README.md" || !name.ends_with(".md") {
                continue;
            }
            count += 1;
            let id_ok =
                name.split_once('-').is_some_and(|(issue, slug)| !issue.is_empty() && issue.chars().all(|c| c.is_ascii_digit()) && slug.len() > 3);
            if !id_ok {
                problems.push(format!("{dir}/{name}: name must be <issue>-<branch-slug>.md"));
            }
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.len() != 1 || !lines.first().is_some_and(|l| l.starts_with("- ")) {
                problems.push(format!("{dir}/{name}: must hold exactly one `- ` bullet entry"));
            }
        }
    }
    if problems.is_empty() {
        println!("progress: ok ({count} fragments)");
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

fn layers() -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = Command::new(cargo).args(["metadata", "--format-version", "1"]).current_dir(root()).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let members: Vec<String> = meta["workspace_members"].as_array().into_iter().flatten().filter_map(|m| m.as_str().map(str::to_owned)).collect();
    let mut problems = Vec::new();
    for package in meta["packages"].as_array().into_iter().flatten() {
        let (Some(id), Some(name)) = (package["id"].as_str(), package["name"].as_str()) else { continue };
        if !members.iter().any(|m| m == id) || UI_CRATES.contains(&name) || name == "xtask" {
            continue;
        }
        for dep in package["dependencies"].as_array().into_iter().flatten() {
            if let Some(dep_name) = dep["name"].as_str()
                && UI_DEPS.contains(&dep_name)
            {
                problems.push(format!("{name} depends on {dep_name}"));
            }
        }
    }
    if problems.is_empty() {
        println!("layers: ok");
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn assets() -> Result<(), String> {
    let root = root();
    let index = std::fs::read_to_string(root.join("ATTRIBUTION.md")).map_err(|e| format!("ATTRIBUTION.md: {e}"))?;
    let mut all = Vec::new();
    files(&root.join("assets"), &mut all);
    files(&root.join("docs/images"), &mut all);
    let missing: Vec<String> = all
        .iter()
        .filter_map(|p| p.strip_prefix(&root).ok())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|p| !index.contains(&format!("`{p}`")))
        .collect();
    if missing.is_empty() {
        println!("assets: ok ({} files)", all.len());
        Ok(())
    } else {
        Err(format!("not in ATTRIBUTION.md:\n{}", missing.join("\n")))
    }
}
