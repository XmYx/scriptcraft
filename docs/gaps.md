# Gaps (ranked)

Ranked by what a working screenwriter misses first, from [parity-checklist.md](parity-checklist.md)
(`cargo xtask parity`). Parity: 81% weighted (P0 92%, P1 82%, P2 47%).

1. **Selection and copy/paste across elements** (P1). Today selection lives inside one element;
   selecting a passage of several elements, cutting and pasting it (as Fountain on the clipboard)
   is the most-used editing operation still missing.
2. **Title page layout** (P0, partial). Key/value fields print in a fixed arrangement; writers expect
   free placement and more fields (WGA registration, revision line).
3. **FDX fidelity** (P0, partial). Open and save styled text (bold, italic, underline runs), script
   notes, scene summaries and revision data, and page-layout settings.
4. **Bold, italic and underline shown as styles** (P2, partial). Fountain emphasis is kept as text;
   render it in the editor, preview and PDF.
5. **Anchored notes and comments** (P1, partial). Notes attached to a range of text, shown in the
   margin, kept in Fountain as inline notes and in FDX as script notes.
6. **Outline by sections** (P1, partial). Show `#` sections and sequences as an outline beside the
   navigator and as groups on the index cards.
7. **Translations** (P1, partial). The ScriptCraft catalog in craft-i18n (`i18n/source-strings.tsv`
   lists the 282 source texts).
8. **Accessibility depth** (P1, partial). Caret and selection reporting on script lines beyond egui's
   text fields; a screen-reader pass; high-contrast theme check.
9. **Locked pages (A pages)** (P2). Keep page breaks after locking; new material goes on 12A pages.
10. **Custom keyboard shortcuts** (P2) through craft-libs `craft-keymap`.
11. **Breakdown tagging** (P2). Tag props, cast and locations in scenes for scheduling exports.
12. **Rename a character everywhere** (P2, partial; Find and Replace with Match case works today).
13. **Go to scene or page** (P2, partial; navigator and cards today).
14. **Scene report** (P2, partial; navigator, cards and the JSON scene list today).
15. **Custom margins and element styles** (P2).
16. **Import from word-processor files and PDF** (P2).
17. **Web and mobile builds** (P2): the egui web target and craft-libs `craft-web-host`.
18. **Real-time co-writing** (P2): craft-contrib RFC-0020.

Also open, outside the parity table: the editor lists the two speeches of dual dialogue one after
the other (print and preview are side by side), and the user dictionary lasts for the session only.
