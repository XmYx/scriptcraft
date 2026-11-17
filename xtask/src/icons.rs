//! `cargo xtask icons`: render `assets/app-icon/scriptcraft.svg` to the PNGs the packages and the
//! window use: `assets/app-icon/hicolor/<n>x<n>/apps/ai.storyteller.scriptcraft.png`, the
//! scalable copy and `assets/app-icon/scriptcraft-1024.png`. The outputs are committed.

use std::path::Path;

use resvg::{tiny_skia, usvg};

const ID: &str = "ai.storyteller.scriptcraft";
const SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];

fn render(tree: &usvg::Tree, size: u32, out: &Path) -> Result<(), String> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or("bad icon size")?;
    let scale = size as f32 / tree.size().width().max(1.0);
    resvg::render(tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    pixmap.save_png(out).map_err(|e| format!("{}: {e}", out.display()))
}

pub fn run(root: &Path) -> Result<(), String> {
    let dir = root.join("assets/app-icon");
    let svg = std::fs::read(dir.join("scriptcraft.svg")).map_err(|e| e.to_string())?;
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    for size in SIZES {
        render(&tree, size, &dir.join(format!("hicolor/{size}x{size}/apps/{ID}.png")))?;
    }
    render(&tree, 1024, &dir.join("scriptcraft-1024.png"))?;
    let scalable = dir.join(format!("hicolor/scalable/apps/{ID}.svg"));
    if let Some(parent) = scalable.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&scalable, &svg).map_err(|e| e.to_string())?;
    write_ico(&dir)?;
    println!("icons: wrote {} PNGs, the scalable SVG and scriptcraft.ico", SIZES.len() + 1);
    Ok(())
}

/// `assets/app-icon/scriptcraft.ico` for Windows (the exe's icon, the installer, Apps and
/// features): the 16 to 256 px hicolor PNGs stored as PNG entries, which Windows Vista and later
/// read directly.
fn write_ico(dir: &Path) -> Result<(), String> {
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|s| std::fs::read(dir.join(format!("hicolor/{s}x{s}/apps/{ID}.png"))).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (size, png) in sizes.iter().zip(&images) {
        let side = if *size >= 256 { 0 } else { *size as u8 };
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in &images {
        out.extend_from_slice(png);
    }
    std::fs::write(dir.join("scriptcraft.ico"), out).map_err(|e| e.to_string())
}
