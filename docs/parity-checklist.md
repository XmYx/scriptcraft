# Parity checklist

The features working screenwriters rely on in the industry's desktop and web screenwriting
applications (the reference app named in [target-app-parity.md](target-app-parity.md) and four
other widely used ones), described in our own words from public documentation and observed
behaviour. `cargo xtask parity` scores this table.

- **Priority:** P0 = a working screenwriter misses it on day one (weight 3); P1 = misses it within
  a project (weight 2); P2 = nice to have (weight 1).
- **State:** done = 1, partial = 0.5, stub or missing = 0.
- **Rank:** the order the gaps are worked on (lower first), by priority and then by how much
  writing it blocks; `-` for done rows.

| Area | Feature | Priority | State | Rank | Notes |
|---|---|---|---|---|---|
| Formatting | Standard element indents, widths and spacing | P0 | done | - | docs/formatting.md |
| Formatting | US Letter and A4 | P0 | done | - | |
| Formatting | (MORE) and (CONT'D) when a speech crosses a page | P0 | done | - | |
| Formatting | Automatic (CONT'D) when the same character speaks again | P1 | done | - | Format ▸ Automatic (CONT'D), on by default |
| Formatting | Dual dialogue side by side | P1 | done | - | layout, page preview and PDF; the draft editor lists the two speeches one after the other |
| Formatting | Keep headings and cues with what follows | P0 | done | - | |
| Formatting | Scene numbers in both margins, locked numbers printed | P0 | done | - | locked numbers print as written |
| Formatting | Page numbers and headers | P0 | done | - | |
| Formatting | Title page | P0 | partial | 2 | key/value fields; no free layout |
| Formatting | Custom margins and element styles | P2 | missing | 15 |  |
| Formatting | Forced page breaks | P2 | done | - | |
| Files | Fountain open and save | P0 | done | - | lossless |
| Files | FDX open | P0 | partial | 3 | elements, numbers, dual dialogue, title page, page breaks; no styles or notes |
| Files | FDX save | P0 | partial | 3 | everything FDX open reads; outline and notes left out (reported) |
| Files | PDF export with exact pagination | P0 | done | - | craft-pdf, font embedded; the layout's pages |
| Files | Plain text export | P2 | done | - | |
| Files | Import from word-processor files and PDF | P2 | missing | 16 |  |
| Writing | Enter/Tab element flow | P0 | done | - | |
| Writing | Auto-complete of names, places, times, transitions | P0 | done | - | |
| Writing | Undo and redo | P0 | done | - | |
| Writing | Find and replace | P0 | done | - | Ctrl+F, Ctrl+H, F3 |
| Writing | Spell-check | P0 | done | - | craft-proof: English always, other languages with their dictionary; F7, add word |
| Writing | Notes and comments | P1 | partial | 5 | Fountain notes shown; no anchored comments |
| Writing | Selection and copy/paste across elements | P1 | missing | 1 |  |
| Writing | Bold, italic and underline shown as styles | P2 | partial | 4 | markup kept as text |
| Writing | Rename a character everywhere | P2 | partial | 12 | Find and Replace with Match case |
| Structure | Scene navigator | P0 | done | - | |
| Structure | Index cards for scenes | P1 | done | - | View ▸ Index Cards: synopsis on the card, keyboard reordering |
| Structure | Outline by sections and sequences | P1 | partial | 6 | sections kept, not shown as outline |
| Structure | Reorder scenes | P1 | done | - | index cards and Alt+↑/↓ |
| Structure | Go to scene or page | P2 | partial | 13 | navigator and cards |
| Production | Lock scene numbers, A/B numbers for new scenes | P1 | done | - | Production ▸ Lock Scene Numbers |
| Production | Revision mode with marks and colours | P1 | done | - | marks in the margin, coloured pages in preview, colour named in PDF headers |
| Production | Locked pages (A pages) | P2 | missing | 9 |  |
| Production | Breakdown tagging | P2 | missing | 11 |  |
| Production | Scene list for storyboards and scheduling | P1 | done | - | docs/interop.md |
| Reports | Character report | P1 | done | - | Script ▸ Reports, scriptcraft-cli report |
| Reports | Location report | P1 | done | - |  |
| Reports | Scene report | P2 | partial | 14 | navigator, cards and JSON |
| Reports | Script statistics | P1 | done | - | CLI pages and scenes only |
| Safety | Autosave and recovery after a crash | P0 | done | - | every minute; offered back on open |
| Safety | Backup of the previous version on save | P1 | done | - | .<name>.backup |
| Environment | Distraction-free (focus) mode | P2 | done | - | F11 |
| Environment | Typewriter scrolling | P2 | done | - |  |
| Environment | Dark page theme | P2 | done | - |  |
| Environment | Zoom | P2 | done | - | |
| Environment | Page preview | P1 | done | - | |
| Environment | Interface translations | P1 | partial | 7 | craft-i18n wired, no catalogs |
| Environment | Screen reader and keyboard access | P1 | partial | 8 | names, live regions, 16 kittest tests |
| Environment | Custom keyboard shortcuts | P2 | missing | 10 | craft-keymap |
| Collaboration | Real-time co-writing | P2 | missing | 18 |  |
| Collaboration | Mobile and web editions | P2 | missing | 17 | egui web build not set up |

