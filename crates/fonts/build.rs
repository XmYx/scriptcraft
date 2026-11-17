//! Optional craft-fonts build input (https://github.com/storytold/craft-fonts).
//!
//! With `CRAFT_FONTS_DIR=<absolute path of a craft-fonts checkout>`, the Liberation Mono faces
//! listed in its `fonts/latin-manifest.txt` are embedded as `SCREENPLAY_FONTS`. Unset, the table
//! is empty and the app draws scripts with egui's built-in monospace face. With
//! `CRAFT_FONTS_REQUIRED=1` a missing checkout or face is a build error (release builds).
//!
//! The manifest is parsed with craft-libs' `craft_fonts_build::parse_manifest`. `Embed` reads
//! only `fonts/manifest.txt`, and Liberation Mono is in the opt-in Latin manifest, so the
//! selection is done here; see `docs/fonts.md`. Reads only the local checkout: no network.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const FAMILY: &str = "Liberation Mono";
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let required = std::env::var("CRAFT_FONTS_REQUIRED").is_ok_and(|v| v == "1");
    let dir = std::env::var_os("CRAFT_FONTS_DIR").filter(|d| !d.is_empty()).map(PathBuf::from);
    let faces = match dir {
        Some(dir) => match faces(&dir) {
            Ok(faces) => faces,
            Err(e) if required => {
                println!("cargo::error={e}");
                Vec::new()
            }
            Err(e) => {
                println!("cargo::warning=screenplay font not embedded: {e}");
                Vec::new()
            }
        },
        None if required => {
            println!("cargo::error=CRAFT_FONTS_REQUIRED=1 but CRAFT_FONTS_DIR is not set");
            Vec::new()
        }
        None => Vec::new(),
    };
    let mut out = String::from("/// Liberation Mono faces from craft-fonts; empty without `CRAFT_FONTS_DIR`.\n");
    out.push_str("pub static SCREENPLAY_FONTS: &[craft_fonts_build::EmbeddedFace] = &[\n");
    for (style, path) in &faces {
        let _ = writeln!(
            out,
            "    craft_fonts_build::EmbeddedFace {{ family: {FAMILY:?}, style: {style:?}, scripts: &[\"Latn\"], index: 0, bytes: include_bytes!({:?}) }},",
            path.display().to_string()
        );
    }
    out.push_str("];\n");
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        println!("cargo::error=OUT_DIR is not set");
        return;
    };
    if let Err(e) = std::fs::write(Path::new(&out_dir).join("craft_fonts.rs"), out) {
        println!("cargo::error=could not write craft_fonts.rs: {e}");
    }
}

/// (style, absolute file) for every Liberation Mono face in the Latin manifest.
fn faces(dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let manifest = dir.join("fonts/latin-manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let size = std::fs::metadata(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?.len();
    if size > MAX_MANIFEST_BYTES {
        return Err(format!("{} is larger than {MAX_MANIFEST_BYTES} bytes", manifest.display()));
    }
    let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let entries = craft_fonts_build::parse_manifest(&text).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let root = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in entries.into_iter().filter(|e| e.family == FAMILY) {
        let file = root.join(&entry.file).canonicalize().map_err(|e| format!("{}: {e}", entry.file))?;
        if !file.starts_with(&root) {
            return Err(format!("{} is outside the craft-fonts checkout", entry.file));
        }
        println!("cargo::rerun-if-changed={}", file.display());
        out.push((entry.style, file));
    }
    if out.is_empty() {
        return Err(format!("{} lists no {FAMILY} face", manifest.display()));
    }
    Ok(out)
}
