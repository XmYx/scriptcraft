# ScriptCraft roadmap

**Stage: skeleton** · next: alpha

> **Last updated:** 2026-11-17 · **Change:** major

ScriptCraft writes screenplays: the first step of the Crafting Apps film pipeline (ScriptCraft →
BoardCraft → FilmCraft). This page is the summary; the ranked work list is
[docs/gaps.md](docs/gaps.md), parity in [docs/target-app-parity.md](docs/target-app-parity.md),
the current focus in [docs/roadmap.md](docs/roadmap.md).

## Where it stands

| Area | State |
|---|---|
| Document model (all screenplay elements, title page) | done, tested |
| Fountain open/save, lossless round trip | done, tested (fixtures, mutation fuzz) |
| FDX import | done for elements, scene numbers, dual dialogue, title page; styles and notes dropped |
| Formatting and pagination (US Letter, A4, MORE/CONT'D, keep-with-next) | done, tested |
| Editor: Enter/Tab element flow, auto-complete, navigator, page preview | working; caret and keyboard model need hands-on polish |
| Scene list for BoardCraft (JSON, stable scene ids) | done, tested, [documented](docs/interop.md) |
| Command-line tool | scenes, convert, pages, stats, commands |
| PDF export | **not started** (planned on craft-libs `craft-pdf`) |
| FDX export | **not started** |
| Accessibility | named script lines, live element announcements, page-preview text nodes; 7 kittest keyboard tests with the accessibility check on |
| Localization | craft-i18n wired, every interface string translatable, 169 source texts listed; no ScriptCraft catalog in craft-i18n yet ([details](docs/localization-parity.md)) |
| Packaging | `packaging/env.sh`, Linux nfpm config and desktop file, hicolor icons; release through craft-common-actions; launcher draft in [docs/launcher-entry.toml](docs/launcher-entry.toml) |

## Alpha gate

A writer can draft, revise and deliver a short script without leaving ScriptCraft:

1. Type a script from scratch with the keyboard only (done).
2. Open a script from FDX or Fountain and save it as Fountain without losing anything (done for
   what is imported).
3. **Export PDF** with correct pagination and an embedded font.
4. **Save FDX** for exchange with other screenwriting software.
5. Every control named and reachable in the accessibility tree, with kittest coverage.

## Next steps

See [docs/gaps.md](docs/gaps.md).

## Progress log

New entries are fragments in `docs/progress.d/` (RFC-0017); the joiner moves them here.
