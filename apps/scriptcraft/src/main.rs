//! ScriptCraft desktop app.
//!
//! Usage: `scriptcraft [options] [script.fountain | script.fdx]`. `--help` lists the options.
//!
//! `--screenshot <file.png>` renders the window, writes it to the file and quits; the images in
//! `docs/images/` are made this way from the bundled sample script.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable))]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use scriptcraft_layout::Paper;
use scriptcraft_ui_egui::{ScriptApp, Startup};

/// Reverse-DNS app id: Linux `.desktop` file, hicolor icon name and Wayland app id.
const APP_ID: &str = "ai.storyteller.scriptcraft";

const USAGE: &str = "\
Usage: scriptcraft [options] [script.fountain | script.fdx]

Options:
  --sample              open the bundled sample script
  --preview             start in page preview
  --paper <us-letter|a4>
  --focus <element>     put the caret in this element (0-based)
  --screenshot <file>   render the window to a PNG file and quit
  --size <W>x<H>        window size in points (default 1440x900)
  -h, --help            show this help
  -V, --version         show the version
";

struct Cli {
    startup: Startup,
    screenshot: Option<PathBuf>,
    size: [f32; 2],
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Cli>, String> {
    let mut cli = Cli { startup: Startup::default(), screenshot: None, size: [1440.0, 900.0] };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("scriptcraft {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--sample" => cli.startup.sample = true,
            "--preview" => cli.startup.preview = true,
            "--paper" => {
                let v = value("--paper")?;
                cli.startup.paper = Paper::from_id(&v).ok_or_else(|| format!("unknown paper size {v:?} (us-letter or a4)"))?;
            }
            "--focus" => {
                let v = value("--focus")?;
                cli.startup.focus_element = Some(v.parse().map_err(|_| format!("--focus needs an element number, not {v:?}"))?);
            }
            "--screenshot" => cli.screenshot = Some(PathBuf::from(value("--screenshot")?)),
            "--size" => {
                let v = value("--size")?;
                let parsed = v.split_once('x').and_then(|(w, h)| Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?]));
                match parsed {
                    Some([w, h]) if (400.0..=8000.0).contains(&w) && (300.0..=8000.0).contains(&h) => cli.size = [w, h],
                    _ => return Err(format!("--size needs WIDTHxHEIGHT, e.g. 1440x900, not {v:?}")),
                }
            }
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            file => cli.startup.file = Some(PathBuf::from(file)),
        }
    }
    Ok(Some(cli))
}

fn main() -> eframe::Result {
    let cli = match parse_args(std::env::args().skip(1)) {
        Ok(Some(cli)) => cli,
        Ok(None) => return Ok(()),
        Err(e) => {
            eprintln!("scriptcraft: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    install_panic_hook();
    let mut viewport =
        egui::ViewportBuilder::default().with_title("ScriptCraft").with_app_id(APP_ID).with_inner_size(cli.size).with_min_inner_size([720.0, 480.0]);
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("ScriptCraft", options, Box::new(move |cc| Ok(Box::new(Shell::new(&cc.egui_ctx, cli)))))
}

/// The app icon (`assets/app-icon`, rendered by `cargo xtask icons`); none if it can't be decoded.
fn window_icon() -> Option<egui::IconData> {
    const PNG: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/ai.storyteller.scriptcraft.png");
    let image = image::load_from_memory_with_format(PNG, image::ImageFormat::Png).ok()?.into_rgba8();
    Some(egui::IconData { width: image.width(), height: image.height(), rgba: image.into_raw() })
}

/// Panics are logged to stderr with their location; the UI pass also catches them (see `Shell::ui`).
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("scriptcraft: internal error: {info}");
        default(info);
    }));
}

/// The window: the app, the last-resort panic guard and the screenshot driver.
struct Shell {
    app: ScriptApp,
    crashed: Option<String>,
    screenshot: Option<PathBuf>,
    frames_until_shot: u32,
    shot_done: Arc<AtomicBool>,
}

impl Shell {
    fn new(ctx: &egui::Context, cli: Cli) -> Shell {
        Shell {
            app: ScriptApp::new(ctx, cli.startup),
            crashed: None,
            screenshot: cli.screenshot,
            frames_until_shot: 12,
            shot_done: Arc::new(AtomicBool::new(false)),
        }
    }

    fn screenshot_step(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        if self.shot_done.load(Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint();
        if self.frames_until_shot == 0 {
            return;
        }
        self.frames_until_shot -= 1;
        if self.frames_until_shot > 0 {
            return;
        }
        let done = self.shot_done.clone();
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |img| {
            let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
            let result = image::RgbaImage::from_raw(img.size[0] as u32, img.size[1] as u32, rgba)
                .ok_or_else(|| "the screenshot has the wrong size".to_owned())
                .and_then(|i| i.save(&path).map_err(|e| e.to_string()));
            match result {
                Ok(()) => println!("screenshot written to {}", path.display()),
                Err(e) => eprintln!("scriptcraft: screenshot failed: {e}"),
            }
            done.store(true, Ordering::SeqCst);
        })));
    }
}

impl eframe::App for Shell {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.crashed.is_none() {
            self.app.logic(ctx);
        }
        self.screenshot_step(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(message) = &self.crashed {
            egui::CentralPanel::default_margins().show(ui, |ui| {
                ui.heading("ScriptCraft hit an internal error");
                ui.label(message.as_str());
                ui.label("Your script is still open. Save it with File ▸ Save As in the next window, or press Continue.");
            });
            if ui.button("Continue").clicked() {
                self.crashed = None;
            }
            return;
        }
        let app = &mut self.app;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.ui(ui)));
        if let Err(payload) = result {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            self.crashed = Some(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Option<Cli>, String> {
        parse_args(list.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn options() {
        let cli = args(&["--sample", "--paper", "a4", "--focus", "7", "--size", "1600x1000", "x.fountain"]).unwrap().unwrap();
        assert!(cli.startup.sample);
        assert_eq!(cli.startup.paper, Paper::A4);
        assert_eq!(cli.startup.focus_element, Some(7));
        assert_eq!(cli.size, [1600.0, 1000.0]);
        assert_eq!(cli.startup.file, Some(PathBuf::from("x.fountain")));
    }

    #[test]
    fn bad_options_are_errors() {
        assert!(args(&["--paper", "legal"]).is_err());
        assert!(args(&["--focus"]).is_err());
        assert!(args(&["--size", "9x9"]).is_err());
        assert!(args(&["--frobnicate"]).is_err());
    }
}
