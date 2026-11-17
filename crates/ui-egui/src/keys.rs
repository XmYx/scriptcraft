//! Exact key matching. egui's `consume_key` ignores extra Shift and Alt, but Enter and Shift+Enter
//! (and Tab and Shift+Tab) do different things in the editor.

use egui::{Event, Key, Ui};

/// Consume a press of `key` with exactly these modifiers (no Ctrl, Cmd or Alt; Shift as given).
pub fn take(ui: &Ui, key: Key, shift: bool) -> bool {
    ui.input_mut(|i| {
        let mut hit = false;
        i.events.retain(|e| {
            let is = matches!(e, Event::Key { key: k, pressed: true, modifiers: m, .. }
                if *k == key && m.shift == shift && !m.ctrl && !m.command && !m.mac_cmd && !m.alt);
            hit |= is;
            !is
        });
        hit
    })
}

/// Whether `key` is pressed this frame with exactly these modifiers, without consuming it.
pub fn peek(ui: &Ui, key: Key, shift: bool) -> bool {
    ui.input(|i| {
        i.events.iter().any(|e| {
            matches!(e, Event::Key { key: k, pressed: true, modifiers: m, .. }
                if *k == key && m.shift == shift && !m.ctrl && !m.command && !m.mac_cmd && !m.alt)
        })
    })
}

/// Take the first event matching `pick` out of this frame's input.
pub fn take_event<T>(ui: &Ui, pick: impl Fn(&Event) -> Option<T>) -> Option<T> {
    ui.input_mut(|i| {
        let at = i.events.iter().position(|e| pick(e).is_some())?;
        let event = i.events.remove(at);
        pick(&event)
    })
}
