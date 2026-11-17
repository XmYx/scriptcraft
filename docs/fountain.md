# Fountain in ScriptCraft

ScriptCraft opens and saves [Fountain](https://fountain.io), written from its public syntax page
(version 1.1). `crates/fountain` is the implementation; its tests are the specification below.

## Reading

Paragraphs are runs of non-blank lines. A line of two or more spaces inside a paragraph keeps the
paragraph going (Fountain's two-space rule). The first line decides what a paragraph is:

| First line | Element |
|---|---|
| `Title:`, `Author:`… at the very start, one of them a known key | Title page (indented lines continue a value) |
| `===` | Page break |
| `#`, `##`… | Section (level = number of `#`) |
| `=` | Synopsis |
| `~` | Lyric |
| `/*` … `*/` (may span blank lines) | Omitted text |
| `[[…]]` (the whole paragraph) | Note |
| `!` | Forced action |
| `@` | Forced character, followed by its speech |
| `.` (not `..`) | Forced scene heading |
| `> … <` | Centered |
| `>` | Forced transition |
| `INT`, `EXT`, `EST`, `INT./EXT`, `INT/EXT`, `I/E` + `.` or space, alone | Scene heading; `#12A#` at the end is the scene number |
| capitals ending in `TO:`, alone | Transition |
| capitals (name before any `(`), with more lines | Character, then parentheticals `( … )`, dialogue and `~` lyrics; a trailing `^` is dual dialogue |
| capitals starting with a shot word (`CLOSE ON`, `ANGLE ON`, `POV`, `INSERT`…), alone | Shot (ScriptCraft's rule; Fountain has no shot mark) |
| anything else | Action (indentation kept) |

Emphasis (`*italic*`, `**bold**`, `_underline_`) and inline notes stay in the text as written.

## Writing

Each element is written in its natural form when that reads back as the same element, and with a
forcing mark (`.`, `!`, `@`, `>`) when it would not: a lowercase character name gets `@`, an action
paragraph in capitals followed by more lines gets `!`. The result: **for every script ScriptCraft
reads, writing it and reading it back gives the same script**, and writing is idempotent. The
sample script is written back byte for byte.

What Fountain cannot carry, and how ScriptCraft writes it:

- A **shot** that does not start with a shot word is written in capitals and reads back as action.
- **Dialogue or a parenthetical without a character** above it is written as action.
- Two dialogue elements in a row in one speech read back as one.
- Trailing spaces on lines are not kept.
