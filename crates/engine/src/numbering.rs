//! Locked scene numbers. Once a production locks the numbers, a scene added later is numbered
//! after the scene before it (`4A`, `4B`, and `A1` before the first), so no other scene's number
//! changes.

use scriptcraft_model::{ElementKind, Screenplay};

/// Lock: give every scene heading without a number one. With no numbers yet the scenes are
/// numbered 1, 2, 3…; afterwards new scenes get letter suffixes. Returns how many were numbered.
pub fn lock(sp: &mut Screenplay) -> usize {
    let headings: Vec<usize> = sp.scene_heading_indices().collect();
    let any_locked = headings.iter().any(|&h| sp.elements.get(h).is_some_and(|e| e.scene_number.is_some()));
    let mut added = 0;
    if !any_locked {
        for (n, &h) in headings.iter().enumerate() {
            if let Some(e) = sp.elements.get_mut(h) {
                e.scene_number = Some((n + 1).to_string());
                added += 1;
            }
        }
        return added;
    }
    // New scenes take letters after the last number that was already locked, so two new scenes
    // after 4 become 4A and 4B (not 4A and 4AA).
    let mut previous: Option<String> = None;
    let mut used: Vec<String> = headings.iter().filter_map(|&h| sp.elements.get(h).and_then(|e| e.scene_number.clone())).collect();
    for &h in &headings {
        let Some(e) = sp.elements.get_mut(h) else { continue };
        match &e.scene_number {
            Some(n) => previous = Some(n.clone()),
            None => {
                let number = next_after(previous.as_deref(), &used);
                used.push(number.clone());
                e.scene_number = Some(number);
                added += 1;
            }
        }
    }
    added
}

/// Clear every scene number.
pub fn unlock(sp: &mut Screenplay) -> usize {
    let mut n = 0;
    for e in sp.elements.iter_mut().filter(|e| e.kind == ElementKind::SceneHeading) {
        if e.scene_number.take().is_some() {
            n += 1;
        }
    }
    n
}

/// The first free number after `previous`: `4` → `4A` → `4B`; before the first scene, `A1`.
fn next_after(previous: Option<&str>, used: &[String]) -> String {
    let candidates: Box<dyn Iterator<Item = String>> = match previous {
        Some(p) => Box::new(('A'..='Z').map(move |c| format!("{p}{c}"))),
        None => Box::new(('A'..='Z').map(|c| format!("{c}1"))),
    };
    for c in candidates {
        if !used.contains(&c) {
            return c;
        }
    }
    // 26 inserted scenes after one number: continue with a double suffix.
    format!("{}AA", previous.unwrap_or("0"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::Element;

    fn heading(text: &str) -> Element {
        Element::new(ElementKind::SceneHeading, text)
    }

    fn numbers(sp: &Screenplay) -> Vec<String> {
        sp.scene_heading_indices().filter_map(|h| sp.elements[h].scene_number.clone()).collect()
    }

    #[test]
    fn first_lock_numbers_in_order_then_new_scenes_get_letters() {
        let mut sp = Screenplay { elements: vec![heading("A"), heading("B"), heading("C")], ..Screenplay::default() };
        assert_eq!(lock(&mut sp), 3);
        assert_eq!(numbers(&sp), ["1", "2", "3"]);
        sp.elements.insert(1, heading("A2"));
        sp.elements.insert(2, heading("A3"));
        sp.elements.insert(0, heading("Before"));
        assert_eq!(lock(&mut sp), 3);
        assert_eq!(numbers(&sp), ["A1", "1", "1A", "1B", "2", "3"]);
        assert_eq!(lock(&mut sp), 0);
        assert_eq!(unlock(&mut sp), 6);
        assert!(numbers(&sp).is_empty());
    }
}
