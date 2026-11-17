# Roadmap (detail)

The summary is [ROADMAP.md](../ROADMAP.md). Parity: 82% weighted
([target-app-parity.md](target-app-parity.md)).

## Done in the polish pass (2026-11-17)

PDF export with exact pagination; FDX save; automatic (CONT'D); dual dialogue side by side;
locked scene numbers with A/B numbering; index cards with synopses and keyboard reordering;
revision mode (marks, colours, kept in Fountain); find and replace; spell-check (craft-proof);
statistics, character and location reports; autosave with recovery and backups; focus mode,
typewriter scrolling and a dark page.

## Done in the user-research pass (2026-11-17)

Researched what users of the reference apps complain about and ask for
([user-research.md](user-research.md)). From it: the editor draws only nearby lines (a 90-page
script went from 53 ms to about 1.3 ms a frame), and whole elements can be selected, cut and copied
as Fountain, with multi-paragraph paste turned into typed elements.

## Now: alpha

1. **Anchored notes and comments** (the most-requested feature).
2. **Title page layout**, **mid-element selection** and **FDX fidelity** (styled text, script
   notes, revision data).
3. **Styled text** (bold, italic, underline) in editor, preview and PDF.
4. **Anchored notes** and an **outline of sections**.
5. **Shell on craft-libs**: `craft-menu`, `craft-keymap`, `craft-settings` (recent files, user
   dictionary, page setup and view options kept between sessions).

## Alpha gate

A writer can draft, revise and deliver a script without leaving ScriptCraft: keyboard-only writing
(done), FDX and Fountain in and out (done for elements), PDF out (done), revisions (done),
copy/paste across elements, and every control in the accessibility tree with kittest coverage
(in place for all new flows).

## After alpha

Locked A-pages; breakdown tagging; custom shortcuts; translations; web build; co-writing.
