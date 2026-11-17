# Formatting and pagination

`crates/layout` sets scripts on a fixed-pitch grid: 10 characters per inch across, 6 lines per
inch down (12 pt Courier-metric type). Columns count from the left edge of the paper.

| Element | Starts at | Width | Align | Blank lines before |
|---|---|---|---|---|
| Scene heading, shot | 1.5 in (col 15) | 60 | left | 1 |
| Action | 1.5 in | 60 | left | 1 |
| Character | 3.7 in (col 37) | 38 | left | 1 |
| Parenthetical | 3.1 in (col 31) | 25 | left | 0 |
| Dialogue, lyric | 2.5 in (col 25) | 35 | left | 0 |
| Transition | ends at 7.5 in | 60 | right | 1 |
| Centered | 1.5 in | 60 | center | 1 |

Scene headings, character cues and transitions are printed in capitals.

| Paper | Size | Body | Lines per page |
|---|---|---|---|
| US Letter | 8.5 × 11 in | 1 in top and bottom | 54 |
| A4 | 210 × 297 mm | 1 in top and bottom | 58 |

The text block is the same on both papers; A4 has a slightly narrower right margin.

## Page breaks

- Blank lines at the top of a page are dropped.
- A scene heading or shot needs room for itself and the first two lines of what follows; a
  character cue (or a parenthetical) needs room for two lines of dialogue and `(MORE)`. Otherwise it
  moves to the next page.
- Action splits across pages when at least two lines stay on each side, preferring the end of a
  sentence.
- Dialogue splits the same way, with `(MORE)` under the last line and `NAME (CONT'D)` at the top of
  the next page.
- An element longer than a page is cut where the page ends.
- Page numbers (`2.`) sit at the top right from page 2; the title page is not numbered.
- With *Format ▸ Scene Numbers*, numbers print in both margins of each heading.

- **Automatic (CONT'D):** when the character who spoke last in the scene speaks again, the cue gets
  `(CONT'D)` (Format ▸ Automatic (CONT'D), on by default; a cue that already says CONT'D is left alone).
- **Dual dialogue:** a speech whose cue is marked dual sits beside the speech before it. Left
  column: cue at column 23, parenthetical at 19, dialogue at 15, 28 wide; right column: cue at 55,
  parenthetical at 51, dialogue at 47, 28 wide. The pair is kept on one page.
- **Locked scene numbers** print as written (`4A`); otherwise scenes are numbered in order.
- **Revisions:** a line changed in a revision gets `*` at column 80; a page holding revised lines
  is that revision's colour in page preview, and its PDF header names the revision ("Blue
  Revision").

Not yet: widow and orphan control within action, locked pages, configurable element styles.
