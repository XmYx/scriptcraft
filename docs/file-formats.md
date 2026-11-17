# File formats

| Format | Open | Save | Notes |
|---|---|---|---|
| Fountain (`.fountain`, `.spmd`, `.txt`) | yes | yes | Native format; lossless round trip, see [fountain.md](fountain.md) |
| FDX (`.fdx`) | yes | not yet | Elements, scene numbers, dual dialogue and title page are read. Text styles, revision marks, script notes, scene summaries, page layout, tags and custom element types are not; unknown paragraph types become action and are named in the open message. Opened FDX scripts save as Fountain. |
| Scene list JSON (`.scenes.json`) | – | export | For BoardCraft, see [interop.md](interop.md) |
| Plain text (`.txt`) | – | export | The paginated pages, separated by form feeds |
| PDF | – | export | The layout's pages through craft-libs `craft-pdf`, Liberation Mono embedded (Hack in builds without craft-fonts); revised pages name their revision in the header |

Readers cap files at 64 MiB and treat every byte as hostile; FDX goes through craft-libs'
`craft-xml`, which bounds depth and node counts and expands no entities.

Revision marks are kept in a Fountain file as an omitted-text block at its end (`/* scriptcraft-revisions 1 … */`),
which other Fountain apps keep and never print; see `crates/engine/src/revisions.rs`.

Saving keeps the previous version as `.<name>.backup`; unsaved changes are autosaved every minute to
`.<name>.autosave` (untitled scripts: the system's temporary folder) and offered back on open.
