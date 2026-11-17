# Attribution

Every asset ScriptCraft includes, embeds at build time or shows in its documentation, with its
author, source and licence. The policy is in [AGENTS.md](AGENTS.md) §5. `cargo xtask assets` fails
if a file under `assets/` or `docs/images/` is missing from this table.

ScriptCraft contains no artwork, templates, sample scripts or fonts from any commercial
screenwriting application. The sample screenplay, test fixtures and app icon are original work of
the ScriptCraft contributors.

## In this repository

| Asset | Author | Source | Licence | Used for |
|---|---|---|---|---|
| `assets/samples/the-lantern-keeper.fountain` | ScriptCraft contributors | original work (2026-11-17) | MIT OR Apache-2.0 | Sample script (`--sample`, File ▸ Open Sample Script), test fixture, screenshots |
| `crates/fountain/tests/fixtures/edge-cases.fountain` | ScriptCraft contributors | original work | MIT OR Apache-2.0 | Fountain reader tests |
| `crates/fdx/tests/fixtures/short-scene.fdx` | ScriptCraft contributors | original work, written from the public structure of FDX files | MIT OR Apache-2.0 | FDX import tests |
| `assets/app-icon/scriptcraft.svg`, `assets/app-icon/hicolor/scalable/apps/ai.storyteller.scriptcraft.svg` | ScriptCraft contributors | original work, drawn by hand in SVG (2026-11-17) | MIT OR Apache-2.0 | Placeholder app icon (a script page with a bookmark ribbon) |
| `assets/app-icon/scriptcraft-1024.png`, `assets/app-icon/hicolor/16x16/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/24x24/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/32x32/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/48x48/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/64x64/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/128x128/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/256x256/apps/ai.storyteller.scriptcraft.png`, `assets/app-icon/hicolor/512x512/apps/ai.storyteller.scriptcraft.png` | ScriptCraft contributors | rendered from `assets/app-icon/scriptcraft.svg` by `cargo xtask icons` (resvg) | MIT OR Apache-2.0 | Window icon, Linux desktop icons, packages |
| `assets/app-icon/scriptcraft.ico` | ScriptCraft contributors | the 16 to 256 px PNGs above, packed by `cargo xtask icons` | MIT OR Apache-2.0 | Windows exe icon, installer and Apps and features |
| `docs/images/editor.png`, `docs/images/page-preview.png`, `docs/images/index-cards.png`, `docs/images/revision-mode.png` | ScriptCraft contributors | screenshots of ScriptCraft (`--sample --screenshot`, with `--cards` and `--revision`) showing the bundled sample script, set in Liberation Mono | MIT OR Apache-2.0 (Liberation Mono glyphs: OFL-1.1) | README |
| `docs/images/pdf-export.png` | ScriptCraft contributors | page 3 of the sample exported by `scriptcraft-cli pdf --scene-numbers`, rendered by `cargo xtask pdf-png` (craft-pdf) | MIT OR Apache-2.0 (Liberation Mono glyphs: OFL-1.1) | README |

## Embedded at build time

| Asset | Author | Source | Licence | Used for |
|---|---|---|---|---|
| Liberation Mono Regular, Bold, Italic, Bold Italic | Red Hat, Inc. and contributors | [craft-fonts](https://github.com/storytold/craft-fonts) `fonts/liberation-mono/` (when `CRAFT_FONTS_DIR` is set) | OFL-1.1 | The screenplay typeface |
| Ubuntu Light, Hack, Noto Emoji, emoji-icon-font subset | various | `epaint_default_fonts` crate | Ubuntu-font-1.0, MIT, Bitstream-Vera, OFL-1.1 | Interface text; script fallback face |
