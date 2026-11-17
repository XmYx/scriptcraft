# Fonts

Scripts are set in **Liberation Mono** (SIL Open Font License 1.1), a fixed-pitch face with the
metrics of the common Courier-style faces, from [craft-fonts](https://github.com/storytold/craft-fonts).
No font file lives in this repository.

`crates/fonts/build.rs` reads `CRAFT_FONTS_DIR` (an absolute path to a craft-fonts checkout) and
embeds the Liberation Mono faces listed in its `fonts/latin-manifest.txt`. Unset, nothing is
embedded and the editor uses egui's built-in monospace face (Hack); pagination is computed on the
character grid, so page breaks do not change. `CRAFT_FONTS_REQUIRED=1` turns a missing checkout or
face into a build error, for release builds.

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run -p scriptcraft -- --sample
```

Shared-library gap: craft-libs' `craft_fonts_build::Embed` reads only `fonts/manifest.txt`, and
Liberation Mono is in the opt-in Latin manifest. ScriptCraft therefore parses the Latin manifest
with `craft_fonts_build::parse_manifest` and selects the family itself (DeckCraft does the same
in its own build script). An `Embed::manifest("fonts/latin-manifest.txt")` option in craft-libs
would let both drop that code.
