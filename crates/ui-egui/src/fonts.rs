//! The screenplay font family.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontId};

/// The egui family name the script is drawn in.
pub const SCRIPT_FAMILY: &str = "screenplay";

/// Install the fonts: Liberation Mono from craft-fonts when it was embedded, then egui's own
/// monospace faces. Returns whether Liberation Mono is in use.
pub fn install(ctx: &egui::Context) -> bool {
    let mut defs = FontDefinitions::default();
    let mut names: Vec<String> = Vec::new();
    let embedded = scriptcraft_fonts::face("Regular");
    if let Some(face) = embedded {
        defs.font_data.insert("liberation-mono".to_owned(), Arc::new(FontData::from_static(face.bytes)));
        names.push("liberation-mono".to_owned());
    }
    names.extend(defs.families.get(&FontFamily::Monospace).cloned().unwrap_or_default());
    defs.families.insert(FontFamily::Name(SCRIPT_FAMILY.into()), names);
    ctx.set_fonts(defs);
    embedded.is_some()
}

/// The script font at `size` points.
pub fn script(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SCRIPT_FAMILY.into()))
}
