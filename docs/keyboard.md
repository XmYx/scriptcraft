# Keyboard

On macOS use Cmd for Ctrl. The rules for Enter and Tab are ScriptCraft's own (`crates/engine/src/typing.rs`).

| Keys | What they do |
|---|---|
| Enter | End the element and start the next: scene heading → action, character → dialogue, parenthetical → dialogue, dialogue → action, transition → scene heading, otherwise action. In the middle of an element, split it there. At the start of a non-empty element, open an empty one above. |
| Shift+Enter | A line break inside the element |
| Tab | Change the element: action → character → transition → scene heading → shot → action; dialogue ⇄ parenthetical (a new parenthetical gets its parentheses) |
| Shift+Tab | The same, backwards |
| Backspace at the start | Join the element to the one above (an empty element is taken out) |
| ↑ at the start / ↓ at the end | Move to the element above / below |
| ↑ ↓, Enter or Tab, Esc in a suggestion list | Choose, accept (a character name then goes on to dialogue), hide |
| Ctrl+1 … Ctrl+7 | Scene heading, action, character, parenthetical, dialogue, transition, shot |
| Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S | New, open, save, save as |
| Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y | Undo, redo (typing in one element is one step) |
| Ctrl+Shift+P | Page preview |
| Ctrl+=, Ctrl+- | Zoom |
