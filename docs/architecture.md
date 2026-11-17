# Architecture

```
model ← fountain, fdx, layout ← interchange ← engine ← ui-egui ← apps/scriptcraft
                                               engine ← apps/scriptcraft-cli
fonts ← ui-egui
```

- **model**: `Screenplay` = `TitlePage` + flat `Vec<Element>`; an element is a kind, its text,
  and per-kind fields (locked scene number, dual-dialogue flag, section level). Text is stored as
  typed; capitals are a display rule.
- **fountain**: reader and writer sharing one paragraph classifier, so the writer knows when an
  element needs a forcing mark.
- **fdx**: import through craft-libs `craft-xml`.
- **layout**: element formats, word wrap and pagination into pages of lines on a character grid;
  also the plain-text rendering. The editor's page rules, the page preview, the scene list and
  the CLI all use it.
- **interchange**: the scene list for BoardCraft and stable scene ids.
- **engine**: the open `Document` (undo with typing coalesced per element, a layout cached per
  change), the Enter/Tab/Backspace rules, auto-complete, the command list and file I/O (atomic
  saves, size caps).
- **ui-egui**: the editor draws the script as one sheet with a text field per element at its
  indent and width; key handling happens before each field sees the keys. Page preview paints
  the layout's lines exactly.

Everything below `ui-egui` is tested without a display (`cargo test`).
