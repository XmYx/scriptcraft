# Target-app parity

The reference application for ScriptCraft is **Final Draft** (the desktop screenwriting
application whose FDX format ScriptCraft opens). This is the only document that names it; feature
labels use our own words ([feature-names.md](feature-names.md)). ScriptCraft is not affiliated
with or endorsed by its maker.

Parity is judged by observed behaviour and public descriptions only (clean room, AGENTS.md §2).
States: **done** (works and is tested), **partial**, **stub** (visible, not working), **missing**.
Nothing here is measured by a script yet; it is an honest estimate from the code and tests.

| Area | Reference app | ScriptCraft | State |
|---|---|---|---|
| Elements: scene heading, action, character, parenthetical, dialogue, transition, shot | yes | yes | done |
| General, cast list, act breaks | yes | read as action / centered on import | partial |
| Enter/Tab element flow | yes | own documented rules ([keyboard.md](keyboard.md)) | done |
| SmartType-style completion (characters, places, times, transitions) | yes | yes | done (not configurable) |
| Title page | yes | key/value dialog, printed layout | partial (no free layout) |
| Pagination, (MORE)/(CONT'D), keep-with-next | yes | yes | done |
| Page sizes | Letter, A4, custom | Letter, A4 | partial |
| Dual dialogue | side by side | stored, laid out one after the other | partial |
| Scene numbers in margins, locked numbers | yes | numbers shown, locked numbers kept from files; no A/B numbering of new scenes | partial |
| Navigator / outline | yes | scene list with pages and eighths | partial |
| Index cards / beat board | yes | none (planned with BoardCraft) | missing |
| Revisions, colored pages, locked pages | yes | none | missing |
| Script notes | yes | Fountain notes kept and shown; FDX notes dropped | partial |
| Reports (scene, character, location) | yes | CLI `stats` only | partial |
| Spelling | yes | none (craft-proof planned) | missing |
| Find and replace | yes | none | missing |
| Native file format | FDX | Fountain (open, save, lossless) | done |
| FDX | open, save | open | partial |
| PDF export | yes | menu entry, disabled | stub |
| Fountain | import/export | native | done |
| Plain text export | yes | yes | done |
| Collaboration | yes | none | missing |
| Production tools (breakdown, scheduling hand-off) | add-ons | scene list JSON for BoardCraft | partial |
| Accessibility (screen reader names, keyboard) | platform | AccessKit names, live region, kittest tests | partial |
| Localization | several languages | craft-i18n wired, no catalogs yet | partial |

See [gaps.md](gaps.md) for the ranked work and [roadmap.md](roadmap.md) for the current focus.
