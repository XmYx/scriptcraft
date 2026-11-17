//! `cargo xtask pdf-png <in.pdf> <page> <out.png> [scale]`: render one page (1-based) of a PDF to
//! PNG with craft-pdf's rasterizer, to look at PDF export output (docs/images/pdf-export.png).

use craft_pdf::interpret::InterpreterSettings;
use craft_pdf::raster::{self, Options, RenderCache};

pub fn run(args: &[String]) -> Result<(), String> {
    let [input, page, output, rest @ ..] = args else { return Err("usage: cargo xtask pdf-png <in.pdf> <page> <out.png> [scale]".into()) };
    let scale: f32 = rest.first().and_then(|s| s.parse().ok()).unwrap_or(1.5);
    let page: usize = page.parse().map_err(|_| "page must be a number")?;
    let bytes = std::fs::read(input).map_err(|e| format!("{input}: {e}"))?;
    let pdf = craft_pdf::open_slice(&bytes, None).map_err(|e| format!("{e:?}"))?;
    let pages = pdf.pages();
    let p = pages.get(page.saturating_sub(1)).ok_or_else(|| format!("the PDF has {} pages", pages.len()))?;
    let options = Options { background: [255; 4], scale: [scale, scale], ..Default::default() };
    let r = raster::render(p, &RenderCache::new(), &InterpreterSettings::default(), &options).map_err(|e| format!("{e:?}"))?;
    let image = image::RgbaImage::from_raw(r.width, r.height, r.bytes().to_vec()).ok_or("bad raster")?;
    image.save(output).map_err(|e| e.to_string())?;
    println!("wrote {output} ({}x{})", r.width, r.height);
    Ok(())
}
