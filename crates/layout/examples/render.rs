//! `cargo run -p scriptcraft-layout --example render -- <file.fountain> [a4]`: print the pages.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("usage: render <file.fountain> [a4]")?;
    let paper = if args.next().as_deref() == Some("a4") { scriptcraft_layout::Paper::A4 } else { scriptcraft_layout::Paper::UsLetter };
    let sp = scriptcraft_fountain::parse_bytes(&std::fs::read(path)?)?;
    let layout = scriptcraft_layout::paginate(&sp, &scriptcraft_layout::PageSetup { paper, scene_numbers: true });
    print!(
        "{}",
        scriptcraft_layout::render_text(&layout)
            .replace('\u{c}', "\n=================================== page break ===================================\n")
    );
    Ok(())
}
