# Roadmap (detail)

The summary is [ROADMAP.md](../ROADMAP.md); this page is the current focus and the alpha gate in
detail.

## Now: skeleton → alpha

1. **PDF export** (craft-libs `craft-pdf` with the `write` feature, Liberation Mono embedded):
   draw the layout's pages exactly as the page preview does. Done when the sample prints with the
   same page breaks as `scriptcraft-cli pages`.
2. **FDX save** for the elements FDX import reads, with a round-trip test over the FDX fixture.
3. **Editor polish**: ↑/↓ across wrapped lines, selection and copy/paste across elements, find.
4. **Shell on craft-libs**: `craft-menu` (native macOS menu bar), `craft-keymap`, `craft-settings`
   (recent files, page setup, language), `craft-ui` controls.
5. **Translations**: the ScriptCraft catalog in craft-i18n (`i18n/source-strings.tsv` is the list
   to translate).

## Alpha gate

A writer can draft, revise and deliver a short script without leaving ScriptCraft: keyboard-only
writing (done), FDX and Fountain in (done), Fountain out (done), **PDF out**, **FDX out**, and
every control in the accessibility tree with kittest coverage (started).

## After alpha

Dual dialogue side by side; revisions and colored pages; reports; outline board shared with
BoardCraft; spelling through `craft-proof`.
