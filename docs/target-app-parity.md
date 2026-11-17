# Target-app parity

ScriptCraft is measured against the screenwriting applications working writers use most:
**Final Draft** (the reference; its FDX format is the industry's exchange file), and **WriterDuet**,
**Fade In**, **Highland** and **Celtx**. This is the only document that names them; feature labels
use our own words ([feature-names.md](feature-names.md)). ScriptCraft is not affiliated with or
endorsed by any of their makers. Features were learned from their public documentation and
observed behaviour only (clean room, AGENTS.md §2).

The feature-by-feature table, with priorities, states and the ranked open list, is
[parity-checklist.md](parity-checklist.md); `cargo xtask parity` scores it (P0 = missed on day one,
weight 3; P1 = missed within a project, weight 2; P2 = nice to have, weight 1; done 1, partial ½).

## Score

| Measured | Weighted | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| 2026-11-17, skeleton (`b1cf89b`) | 48% | 64% | 32% | 26% |
| 2026-11-17, polish pass | **81%** | **92%** | **82%** | **47%** |

## Where ScriptCraft stands against each reference

| Area | What the references offer | ScriptCraft |
|---|---|---|
| Formatting and pagination | All five paginate to the spec-script standard; (MORE)/(CONT'D), automatic (CONT'D), dual dialogue side by side | Same rules, tested; dual dialogue side by side in print and preview (the editor lists both speeches in order) |
| Scene numbers and production | Locked numbering with A/B scenes (all five); locked pages and revision sets with coloured pages (the reference, Fade In, WriterDuet) | Locked numbering with A/B scenes; revision marks and colours; **no locked A-pages** |
| FDX | Native in the reference; import and export in the other four | Opens and saves FDX elements, numbers, dual dialogue and the title page; **no styled text, script notes or revision data in FDX yet** |
| PDF | All five | Exact pages from the layout, font embedded |
| Fountain | Native in Highland; import/export in Fade In, WriterDuet and the reference | Native, lossless |
| Writing aids | Auto-complete of names, places, times and transitions (all five); spell-check (all) | Both; spell-check through craft-proof |
| Find and replace | All five | Yes, with match case |
| Structure | Index cards / boards (the reference, WriterDuet, Celtx); outline navigators (all) | Scene navigator; index cards with synopses and keyboard reordering; **no outline of sections** |
| Notes | Anchored comments and script notes (all) | Fountain notes as their own lines; **no anchored comments** |
| Reports and statistics | Scene, character and location reports (the reference, Fade In, Celtx) | Statistics, character and location reports; CLI tables |
| Safety | Autosave and backups (all desktop apps) | Autosave every minute with recovery on open; backup of the previous version on save |
| Environment | Focus / distraction-free and typewriter modes, dark themes (Highland, Fade In, the reference) | Focus mode, typewriter scrolling, dark page |
| Collaboration | Real-time co-writing (WriterDuet, the reference's cloud, Celtx) | **None** |
| Platforms | Desktop, web or mobile depending on the app | Desktop (Linux, macOS, Windows); **no web or mobile build yet** |
| Accessibility | Varies; rarely documented | AccessKit names, live regions, 16 kittest keyboard tests with the accessibility check |
| Languages | Several interface languages | craft-i18n wired; **no ScriptCraft catalogs yet** |

The ranked remaining work is [gaps.md](gaps.md); the current focus is [roadmap.md](roadmap.md).
