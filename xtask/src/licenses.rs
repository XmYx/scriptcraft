//! `cargo xtask licenses`: every dependency's licence is on the Craft allowlist
//! (craft-contrib `standards/house-rules-rust.md`, "Licence allowlist"). An expression passes
//! when every identifier in it is allowed; `MPL-2.0` is allowed for crates used unmodified.
//! A top-level `A OR B` choice passes when one alternative passes on its own; those choices are
//! printed (we take the permissive alternative, e.g. `MIT OR Apache-2.0 OR LGPL-2.1-or-later`).

use std::process::Command;

const ALLOWED: [&str; 21] = [
    "ISC",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Apache-2.0",
    "Unicode-3.0",
    "OFL-1.1",
    "Ubuntu-font-1.0",
    "Bitstream-Vera",
    "CC0-1.0",
    "CC-BY-4.0",
    "CC-BY-SA-4.0",
    "Zlib",
    "BSL-1.0",
    "0BSD",
    "Unlicense",
    "IJG",
    "NCSA",
    "CDLA-Permissive-2.0",
    "LLVM-exception",
    "LicenseRef-Public-Domain",
];
/// Third-party crates used unmodified (never vendored or patched here).
const UNMODIFIED_ONLY: [&str; 1] = ["MPL-2.0"];

/// The identifiers of an SPDX expression (`(MIT OR Apache-2.0) AND Unicode-3.0`).
fn identifiers(expr: &str) -> Vec<String> {
    expr.replace(['(', ')'], " ")
        .replace('/', " OR ")
        .split_whitespace()
        .filter(|w| !matches!(*w, "OR" | "AND" | "WITH"))
        .map(|w| w.trim_end_matches('+').to_owned())
        .collect()
}

pub fn run(root: &std::path::Path) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = Command::new(cargo).args(["metadata", "--format-version", "1"]).current_dir(root).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let mut problems = Vec::new();
    let mut mpl = Vec::new();
    let mut chosen = Vec::new();
    let mut count = 0;
    for package in meta["packages"].as_array().into_iter().flatten() {
        let name = package["name"].as_str().unwrap_or("?");
        let version = package["version"].as_str().unwrap_or("?");
        count += 1;
        let Some(expr) = package["license"].as_str() else {
            problems.push(format!("{name} {version}: no SPDX licence expression"));
            continue;
        };
        let ok = |e: &str| identifiers(e).iter().all(|i| ALLOWED.contains(&i.as_str()) || UNMODIFIED_ONLY.contains(&i.as_str()));
        let ids = identifiers(expr);
        if ok(expr) {
            if ids.iter().any(|i| UNMODIFIED_ONLY.contains(&i.as_str())) {
                mpl.push(format!("{name} {version}"));
            }
        } else if !expr.contains('(') && expr.split(" OR ").any(ok) {
            chosen.push(format!("{name} {version} ({expr})"));
        } else {
            problems.push(format!("{name} {version}: {expr}"));
        }
    }
    if !mpl.is_empty() {
        println!("licenses: MPL-2.0, used unmodified: {}", mpl.join(", "));
    }
    for c in &chosen {
        println!("licenses: permissive alternative taken: {c}");
    }
    if problems.is_empty() {
        println!("licenses: ok ({count} packages)");
        Ok(())
    } else {
        Err(format!("licences outside the allowlist:\n{}", problems.join("\n")))
    }
}
