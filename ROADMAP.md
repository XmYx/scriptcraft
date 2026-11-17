# ScriptCraft roadmap

**Stage: skeleton** · next: alpha

> **Last updated:** 2026-11-17 · **Change:** major

ScriptCraft writes screenplays: the first step of the Crafting Apps film pipeline (ScriptCraft →
BoardCraft → FilmCraft). This page is the summary; the ranked work list is
[docs/gaps.md](docs/gaps.md), parity in [docs/target-app-parity.md](docs/target-app-parity.md),
the current focus in [docs/roadmap.md](docs/roadmap.md).

## Where it stands

**Parity: 82% weighted** (P0 92%, P1 85%, P2 47%; it was 48% at the skeleton), measured against
the industry's screenwriting apps in [docs/target-app-parity.md](docs/target-app-parity.md).
What their users complain about and ask for: [docs/user-research.md](docs/user-research.md).

| Area | State |
|---|---|
| Model, Fountain (lossless), pagination (US Letter, A4, MORE/CONT'D, automatic CONT'D, dual dialogue) | done, tested |
| PDF export (exact pages, font embedded) | done, tested |
| FDX | open and save elements, scene numbers, dual dialogue, title page; no styled text or notes yet |
| Editor: Enter/Tab flow, auto-complete, find and replace, spell-check, navigator, index cards, page preview | done; fast on feature-length scripts (only nearby lines are drawn) |
| Clipboard: whole elements cut and copied as Fountain, multi-paragraph paste becomes elements | done; no selection from mid-element yet |
| Production: locked scene numbers with A/B, revision marks and colours | done; no locked pages |
| Reports and statistics | done |
| Safety: autosave with recovery, backups, atomic saves | done |
| Focus mode, typewriter scrolling, dark page | done |
| Scene list for BoardCraft (JSON, stable scene ids) | done, [documented](docs/interop.md) |
| Accessibility | named script lines and cards, live regions, page text nodes; 16 kittest keyboard tests with the accessibility check |
| Localization | craft-i18n wired, every interface string translatable; no ScriptCraft catalog in craft-i18n yet ([details](docs/localization-parity.md)) |
| Packaging | `packaging/env.sh`, Linux nfpm config and desktop file, hicolor icons; release through craft-common-actions; launcher draft in [docs/launcher-entry.toml](docs/launcher-entry.toml) |

## Next

The ranked list is [docs/gaps.md](docs/gaps.md); first: anchored notes and comments, title page
layout, mid-element selection, FDX fidelity. Detail: [docs/roadmap.md](docs/roadmap.md).

## Progress log

New entries are fragments in `docs/progress.d/` (RFC-0017); the joiner moves them here.
