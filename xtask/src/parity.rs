//! `cargo xtask parity`: score `docs/parity-checklist.md` (weights P0 3, P1 2, P2 1; done 1,
//! partial ½, stub or missing 0) and list the open rows by rank.

use std::path::Path;

pub fn run(root: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(root.join("docs/parity-checklist.md")).map_err(|e| e.to_string())?;
    let (mut got, mut total) = (0.0f64, 0.0f64);
    let mut per_priority = [(0.0f64, 0.0f64); 3];
    let mut open: Vec<(u32, String, String, String)> = Vec::new();
    for line in text.lines().filter(|l| l.starts_with('|') && !l.starts_with("|---") && !l.starts_with("| Area")) {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        let [area, feature, priority, state, rank, ..] = cells.as_slice() else { continue };
        let (p, weight) = match *priority {
            "P0" => (0, 3.0),
            "P1" => (1, 2.0),
            "P2" => (2, 1.0),
            other => return Err(format!("unknown priority {other:?} in: {line}")),
        };
        let value = match *state {
            "done" => 1.0,
            "partial" => 0.5,
            "stub" | "missing" => 0.0,
            other => return Err(format!("unknown state {other:?} in: {line}")),
        };
        got += weight * value;
        total += weight;
        if let Some(slot) = per_priority.get_mut(p) {
            slot.0 += value;
            slot.1 += 1.0;
        }
        if value < 1.0 {
            open.push((rank.parse().unwrap_or(999), (*priority).to_owned(), format!("{area}: {feature}"), (*state).to_owned()));
        }
    }
    if total == 0.0 {
        return Err("no rows".to_owned());
    }
    let pct = |a: f64, b: f64| if b == 0.0 { 0.0 } else { 100.0 * a / b };
    println!(
        "parity: {:.0}% weighted (P0 {:.0}%, P1 {:.0}%, P2 {:.0}%)",
        pct(got, total),
        pct(per_priority[0].0, per_priority[0].1),
        pct(per_priority[1].0, per_priority[1].1),
        pct(per_priority[2].0, per_priority[2].1)
    );
    open.sort();
    for (rank, priority, feature, state) in open {
        println!("  {rank:>3}  {priority}  {feature} ({state})");
    }
    Ok(())
}
