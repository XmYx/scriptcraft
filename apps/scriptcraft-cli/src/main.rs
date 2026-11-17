//! `scriptcraft-cli`: ScriptCraft without a window, for scripts, pipelines and agents.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable))]

use std::path::{Path, PathBuf};

use scriptcraft_engine::{Command, io};
use scriptcraft_layout::{PageSetup, Paper, paginate};
use scriptcraft_model::ElementKind;

const USAGE: &str = "\
Usage: scriptcraft-cli <command> [options]

Commands:
  scenes  <script> [--out <file.json>]   scene list for storyboards (JSON, docs/interop.md)
  convert <script> <out.fountain>        open Fountain or FDX, write Fountain
  pages   <script> [--scene-numbers]     the paginated script as plain text
  stats   <script>                       title, pages, scenes and speaking characters
  commands                               the app's command ids and labels

Options for every command:
  --paper <us-letter|a4>                 paper size for pagination (default us-letter)

<script> is a .fountain or .fdx file. Exit codes: 0 done, 1 failed, 2 bad arguments.
";

struct Args {
    command: String,
    files: Vec<PathBuf>,
    out: Option<PathBuf>,
    setup: PageSetup,
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut it = raw.into_iter();
    let command = it.next().ok_or("no command given")?;
    let mut args = Args { command, files: Vec::new(), out: None, setup: PageSetup::default() };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--paper" => {
                let v = it.next().ok_or("--paper needs a value")?;
                args.setup.paper = Paper::from_id(&v).ok_or_else(|| format!("unknown paper size {v:?}"))?;
            }
            "--scene-numbers" => args.setup.scene_numbers = true,
            "--out" | "-o" => args.out = Some(PathBuf::from(it.next().ok_or("--out needs a file")?)),
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            file => args.files.push(PathBuf::from(file)),
        }
    }
    Ok(args)
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.is_empty() || raw.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return;
    }
    if raw.iter().any(|a| a == "-V" || a == "--version") {
        println!("scriptcraft-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let args = match parse_args(raw) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("scriptcraft-cli: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    match run(&args) {
        Ok(Outcome::Done) => {}
        Ok(Outcome::Usage(e)) => {
            eprintln!("scriptcraft-cli: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("scriptcraft-cli: {e}");
            std::process::exit(1);
        }
    }
}

enum Outcome {
    Done,
    Usage(String),
}

fn script_arg(args: &Args, n: usize) -> Result<&Path, String> {
    args.files.get(n).map(PathBuf::as_path).ok_or_else(|| format!("{} needs a script file", args.command))
}

fn run(args: &Args) -> Result<Outcome, String> {
    let open = |path: &Path| io::open(path).map_err(|e| e.to_string());
    match args.command.as_str() {
        "scenes" => {
            let Ok(path) = script_arg(args, 0) else { return Ok(Outcome::Usage("scenes needs a script file".into())) };
            let opened = open(path)?;
            let source = path.file_name().map(|n| n.to_string_lossy().into_owned());
            let json = io::scene_list_json(&opened.screenplay, &args.setup, source).map_err(|e| e.to_string())?;
            write_or_print(args.out.as_deref(), &json)?;
        }
        "convert" => {
            let (Ok(input), Ok(output)) = (script_arg(args, 0), script_arg(args, 1)) else {
                return Ok(Outcome::Usage("convert needs an input script and an output .fountain file".into()));
            };
            let opened = open(input)?;
            if !opened.unknown_types.is_empty() {
                eprintln!("read as action: {}", opened.unknown_types.join(", "));
            }
            io::save_fountain(output, &opened.screenplay).map_err(|e| e.to_string())?;
        }
        "pages" => {
            let Ok(path) = script_arg(args, 0) else { return Ok(Outcome::Usage("pages needs a script file".into())) };
            let opened = open(path)?;
            write_or_print(args.out.as_deref(), &io::plain_text(&opened.screenplay, &args.setup))?;
        }
        "stats" => {
            let Ok(path) = script_arg(args, 0) else { return Ok(Outcome::Usage("stats needs a script file".into())) };
            let sp = open(path)?.screenplay;
            let layout = paginate(&sp, &args.setup);
            println!("title: {}", sp.title().unwrap_or("(none)"));
            println!("paper: {}", args.setup.paper.label());
            println!("pages: {}", layout.page_count());
            println!("scenes: {}", sp.scene_heading_indices().count());
            println!("elements: {}", sp.elements.len());
            let mut speeches: Vec<(String, usize)> = Vec::new();
            for name in sp.elements.iter().filter(|e| e.kind == ElementKind::Character).filter_map(|e| e.character_name()) {
                match speeches.iter_mut().find(|(n, _)| *n == name) {
                    Some((_, count)) => *count += 1,
                    None => speeches.push((name, 1)),
                }
            }
            speeches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            println!("characters (speeches):");
            for (name, count) in speeches {
                println!("  {name}: {count}");
            }
        }
        "commands" => {
            for c in Command::all() {
                println!("{}\t{}", c.id(), c.label());
            }
        }
        other => return Ok(Outcome::Usage(format!("unknown command {other:?}"))),
    }
    Ok(Outcome::Done)
}

fn write_or_print(out: Option<&Path>, text: &str) -> Result<(), String> {
    match out {
        Some(path) => io::write_atomic(path, text.as_bytes()).map_err(|e| e.to_string()),
        None => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_options() {
        let a = parse_args(vec!["scenes".into(), "x.fountain".into(), "--paper".into(), "a4".into(), "--out".into(), "y.json".into()]).unwrap();
        assert_eq!(a.command, "scenes");
        assert_eq!(a.setup.paper, Paper::A4);
        assert_eq!(a.out, Some(PathBuf::from("y.json")));
        assert!(parse_args(vec!["pages".into(), "--bogus".into()]).is_err());
    }
}
