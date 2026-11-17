<!-- craft-house-rules:start core source=279b2c09ef2b2f0b4f5e616a47d39e29db0a1eb3 sha256=bee0ffafad196b68c76425770e344e588463bbda685ab41581fcbabb85dfc930 -->
## House rules

These apply in every Craft repository. A later rule may be stricter, not weaker. The source is `standards/house-rules-core.md` in `storytold/craft-contrib`. Propose changes there, not in a copy.

1. **A person is accountable.** Whoever runs an agent answers for what it opens and for its review comments.
2. **Check, then claim.** Before writing code, read the issue comments, search pull requests for `#N` and the keywords, and check recent commits on `main`. Comment on the issue first. Release a claim you will not finish.
3. **Verify, don't guess.** Open a definition before using it. Confirm the cause in the code, even when someone names a location.
4. **Keep fixes narrow and tested.** Fix what the issue describes, where the bug is. Add a test that fails without the fix.
5. **Review the diff before opening a pull request.** Identifiers exist as used, the test fails without the fix, nothing unrelated changed, and the description is true.
6. **Look for a better pull request.** If another fixes the same thing, close yours when it is better or equal, and say why.
7. **Report only what ran.** Say what you tested and what you could not run. Without a local build or test, say so and rely on CI before marking ready.
8. **`main` changes through a pull request.** Do not push to it. Branch protection enforces this. There is no exception.
9. **Stop and ask only about** licensing outside the allowlist, publishing, releases, a new remote, secrets, spending money, destructive git, and anything else hard to undo. Otherwise keep working. `plan/` is a gitignored notebook, not rules. Without it, do not ask for it and do not stop.
10. **Keep secrets and personal details out** of the repository, logs and requests.
11. **CI stays shared.** Workflows call `storytold/craft-common-actions`. Do not change CI in a feature or fix pull request. If CI cannot cover a change, open an issue there.
12. **Guards say why.** A check that looks removable says what it prevents. Read that before changing it.
13. **Instructions must be readable by everyone.** No notes about one machine, and no links to files a reader cannot open. `craftrules` is retired. Do not link to it.
14. **Keep pull requests moving.** Concurrent pull requests are fine; unanswered actionable feedback is not. Respond in 7 days, a reminder at 14, closure eligible at 30, from when an actionable request waits on the author. Suggestions and unconfirmed bot findings do not start the clock. A fix, explanation or agreed plan resets it; unrelated activity does not. Waiting on a reviewer or maintainer pauses it and does not count against the author. A maintainer decides closure; it is never automatic. Reuse search results, do not poll, and stop at a rate limit.
15. **No agent attribution.** A pull request does not say an agent wrote it. A repository rule may not require that.
16. **Shared libraries first for Craft apps.** Before adding or changing reusable file-format, codec, protocol or UI code, check `storytold/craft-libs` and relevant open pull requests. Reuse its public APIs or extend the appropriate shared library instead of duplicating an implementation in an app. Keep app-specific policy and model mapping local; preserve behavior, security and resource limits, attribution, layering and regression coverage. Document API gaps and maintainer-approved exceptions rather than silently dropping behavior. This does not authorize dependencies on sibling application crates or override existing platform, licensing or dependency-source requirements.
<!-- craft-house-rules:end -->
<!-- craft-house-rules:start rust source=279b2c09ef2b2f0b4f5e616a47d39e29db0a1eb3 sha256=56ef2b873de68940cf903069bbd0927b119f5c9ec2717f7f627041d2ebe8e4f2 -->
## Rust Craft repositories

These apply to native Craft apps and to shared Rust libraries. Select them by repository type. A repository that merely contains some Rust does not take this block. Rules later in a repository's file may be stricter, but not weaker. The source is `standards/house-rules-rust.md` in [storytold/craft-contrib](https://github.com/storytold/craft-contrib).

1. **Never crash.** Non-test code does not panic and has no `unsafe`, except where the repository file names an exception. Return errors, treat input as hostile, and bound recursion and allocations. New production crate roots copy the `deny` attribute from an existing crate: `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::unimplemented`, `clippy::todo`, `clippy::unreachable`. Every crash fix comes with a test that panicked before the fix.
2. **Clean room.** Learn from observed behaviour and public specifications. Do not copy another product's code, assets, strings or artwork. Check a dependency's licence before adding it.
3. **Assets and fonts.** Every asset is original or suitably licensed, and is recorded in `ATTRIBUTION.md` in the same change. Font files live in [storytold/craft-fonts](https://github.com/storytold/craft-fonts), not in the app repository.
4. **Rust only for native Craft apps.** No handwritten JavaScript or TypeScript, and no Tauri, Electron or webview shell, in those apps. Shared tooling follows its documented technical scope and maintainer-approved exceptions. Craft PR Review's existing Tauri and TypeScript UI is one such exception. It does not relax this rule for native apps. Preserve any existing web build.
5. **Time goes through `chrono`.** Do not hand-roll calendar conversion. For dates beyond what `chrono` can represent, follow [standards/time.md](https://github.com/storytold/craft-contrib/blob/main/standards/time.md).
6. **Contributor credits.** Add or edit only your own person's entry in `contributors/people.toml`, and only after asking them. Never guess a name or hand-edit `contributors.json`.

## Licence allowlist

Dependencies and assets use SPDX identifiers. An expression is allowed when every identifier is in this list. A repository may name a shorter list. It may not add an identifier here by editing its own copy of these rules.

- `ISC`
- `MIT`
- `BSD-2-Clause`
- `BSD-3-Clause`
- `Apache-2.0`
- `Unicode-3.0`
- `OFL-1.1`
- `Ubuntu-font-1.0`
- `Bitstream-Vera`
- `CC0-1.0`
- `CC-BY-4.0`
- `CC-BY-SA-4.0`
- `Zlib`
- `BSL-1.0`
- `0BSD`
- `Unlicense`
- `IJG`
- `NCSA`
- `CDLA-Permissive-2.0`
- `Apache-2.0 WITH LLVM-exception`
- `LicenseRef-Public-Domain`

`MPL-2.0` is allowed for third-party dependencies used unmodified; vendoring, patching or copying its code needs a maintainer's approval. `MPL-2.0+` counts as `MPL-2.0`.

`LicenseRef-Storyteller-Trademark` is not in this list. It covers Storyteller brand marks only, where a repository already says so.
<!-- craft-house-rules:end -->
<!-- craft-house-rules:start accessibility source=279b2c09ef2b2f0b4f5e616a47d39e29db0a1eb3 sha256=9cff0d580a7cf67c32980531963c0041244c28da8f832c501ae5ff4d864ed472 -->
## Accessibility

These apply to native Craft apps and to shared libraries that draw interface. The source is `standards/house-rules-accessibility.md` in `storytold/craft-contrib`. The full rules are [standards/accessibility.md](https://github.com/storytold/craft-contrib/blob/main/standards/accessibility.md).

People use the apps with screen readers, braille displays, magnifiers, high contrast, large text, voice control or only a keyboard. They reach the interface only through its AccessKit tree, keyboard focus and colours. A UI change is not done until it works for them. The bar is WCAG 2.2 AA as applied to software.

1. **Design first.** Name the pattern (menu, tab list, toolbar, tree, grid, list, dialog, slider, canvas) and follow its keyboard model and roles from the WAI-ARIA Authoring Practices Guide and the platform's guidelines. Observe how the reference app exposes it with Accessibility Insights, Accessibility Inspector or Accerciser.
2. **Keyboard.** Every action works from the keyboard, in visual order, with no trap. Esc closes and returns focus. Alt or F10 and F6 behave as on the platform. Move egui's focus (`request_focus`), never only a highlight.
3. **Semantics.** Every control has a role, a translated name that starts with its visible label, and its state and value. Shortcuts and access keys are set on the node, in the current platform's form. A drawn surface publishes nodes for its content, with caret, selection and correct editability.
4. **Feedback.** Announce changes away from focus through a live region.
5. **Seeing.** Contrast at least 4.5:1 for text and 3:1 for controls and focus, in both themes. Never colour alone. Follow the system's high contrast, text size and reduced motion settings.
6. **Pointer.** Anything done by dragging can be done without dragging.
7. **Check the tree, not only the screenshot.** Each new or changed control, dialog or keyboard path gets a kittest test that drives it by keyboard and asserts roles, names, states, focus and announcements. If the helpers do not exist yet, say so in the pull request. Keep `with_accessibility_check` on and CI's platform checks green. With `ui.screenshot`, also read the accessibility tree.
8. **Rule out known failures:** focus left in the document behind a menu, ribbon or KeyTips; editable surfaces reported read-only; dialogs that do not take or return focus, or leave controls behind them reachable; painted rows with no node; identical names; another platform's shortcut text in a name; silent state changes; drag-only actions.
9. **Be honest in the pull request.** Its Accessibility section says what the tests and the tree cover. Tree tests never prove "works with screen readers". Ask for `needs screen reader test` only for an accessibility fix or accessibility logic of its own, and never hold a pull request back for that test.
<!-- craft-house-rules:end -->

# AGENTS.md — rules for every agent and contributor

ScriptCraft is a clean-room, open-source, pure-Rust screenwriting application: write, format and
paginate screenplays, open and save Fountain, open FDX. It is the first step of the Crafting
Apps film pipeline: **ScriptCraft** (script) → **BoardCraft** (storyboards) → **FilmCraft**
(editing). Same conventions as its siblings (WordCraft, DeckCraft, FilmCraft…).

The blocks above are copied from `storytold/craft-contrib` and win over anything below. The rules
below are ScriptCraft's own and may only be stricter. Names of other companies' products follow
[standards/naming.md](https://github.com/storytold/craft-contrib/blob/main/standards/naming.md).

## 1. Never crash

People trust ScriptCraft with their scripts. A malformed Fountain or FDX file, an odd keystroke or
a full disk gives an error the user can act on, never a crash and never lost work.

- No `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!` outside tests; every
  production crate root carries the `deny` attribute. `unsafe_code = "forbid"` for the whole
  workspace; there is no exception.
- Input is hostile: file readers cap their input (64 MiB) and never recurse on file structure
  (`crates/fountain/src/read.rs` reads paragraphs in a loop for that reason). Index with `get`.
- Saving goes through `scriptcraft_engine::io::write_atomic` (temporary file, then rename), so a
  failed save never leaves a half-written script.
- The desktop shell (`apps/scriptcraft`) runs the UI pass under `catch_unwind` and logs panics;
  that guard is a last resort, not a licence.
- Parsers get mutation tests (truncation, byte flips, spliced markers); see
  `crates/fountain/tests/round_trip.rs` and `crates/fdx/tests/import.rs`.

## 2. Clean room and names

- Formats are implemented from public descriptions: Fountain from its published syntax page, FDX
  from the publicly observable structure of the files. Never read another screenwriting
  application's code, and never copy its strings, templates or artwork.
- No other company's product or feature name as a label; factual statements are fine ("opens FDX
  files"). The reference application is named only in
  [docs/target-app-parity.md](docs/target-app-parity.md). New feature names are recorded in
  [docs/feature-names.md](docs/feature-names.md).
- Test fixtures and samples are written for this repository (`assets/samples/`,
  `crates/*/tests/fixtures/`). Never commit a published screenplay, even an excerpt.

## 3. Layering

Nothing below the UI depends on egui, eframe, winit or rfd:

```
model ← fountain, fdx, layout ← interchange ← engine ← ui-egui ← apps/scriptcraft
                                               engine ← apps/scriptcraft-cli
fonts ← ui-egui
```

`cargo xtask layers` checks this. Formatting and pagination live in `crates/layout` and must stay
UI-independent: the page preview, the plain-text export and the scene list all read the same
layout. Every action is a `scriptcraft_engine::Command` with a stable id.

## 4. Shared libraries

Reuse `storytold/craft-libs` before writing format or UI code (house rule 16). In use: `craft-xml`
(FDX), `craft-fonts-build` (the craft-fonts manifest), `craft-i18n` (translations), `craft-pdf`
(PDF export and, in xtask, rasterizing PDFs), `craft-proof` (spelling). Planned: `craft-ui`,
`craft-menu`, `craft-keymap` and `craft-settings` for the shell. Pin every craft-libs crate to the same `rev`. API gaps are written down, not worked around
by copying: [docs/fonts.md](docs/fonts.md) (Latin manifest).

## 5. Fonts, translations and assets

- Fonts come from [craft-fonts](https://github.com/storytold/craft-fonts) through
  `CRAFT_FONTS_DIR` ([docs/fonts.md](docs/fonts.md)); never commit a font file.
- Every user-visible string goes through `crate::i18n` (`tr`, `trf`, `trn`, `tr_command`) with the
  English source text as the key. Translations come from
  [craft-i18n](https://github.com/storytold/craft-i18n) through `CRAFT_I18N_DIR`
  (`crates/ui-egui/build.rs`); `CRAFT_I18N_REQUIRED=1` for releases. After changing interface text,
  run `cargo xtask strings` ([docs/localization-parity.md](docs/localization-parity.md)).
- Every asset is listed in [ATTRIBUTION.md](ATTRIBUTION.md) in the same change. App icons are
  rendered from `assets/app-icon/scriptcraft.svg` by `cargo xtask icons`.

## 6. Accessibility

Every control has a translated accessible name; each script line is named by its element kind
(and scene number on a heading); the element-kind label in the status bar is a live region; the
page preview publishes each page's text as a document node. A new or changed control or keyboard
path gets an egui_kittest test with the accessibility check on (`crates/ui-egui/tests/keyboard.rs`).

## 7. Progress docs and release notes

[ROADMAP.md](ROADMAP.md) is the summary; [docs/gaps.md](docs/gaps.md) the ranked work list;
[docs/target-app-parity.md](docs/target-app-parity.md), [docs/roadmap.md](docs/roadmap.md) and
[docs/localization-parity.md](docs/localization-parity.md) the detail. Per RFC-0017 a change adds
a progress fragment `docs/progress.d/<issue>-<branch-slug>.md` (and, when users notice it, a
release-note fragment in `packaging/release-notes.d/`); it does not edit the status line or the
log heading.

## 8. Commands and gates

```sh
cargo build                     # everything
cargo test                      # all tests, including kittest UI tests (no display needed)
cargo xtask ci                  # fmt --check, clippy -D warnings, tests, layers, assets,
                                # licences, source strings, progress fragments
cargo xtask parity              # score docs/parity-checklist.md, list open gaps by rank
cargo xtask pdf-png <pdf> <page> <png>   # look at PDF export output
cargo run -p scriptcraft -- --sample
CRAFT_FONTS_DIR=$PWD/../craft-fonts CRAFT_I18N_DIR=$PWD/../craft-i18n cargo run -p scriptcraft -- --sample
cargo run -p scriptcraft-cli -- scenes assets/samples/the-lantern-keeper.fountain
```

CI is `.github/workflows/`, calling `storytold/craft-common-actions`; don't change it in a feature
pull request. The version lives only in `[workspace.package]`; `packaging/env.sh` reads it.
Releases come from the maintainers' `release` branch; don't create release branches or tags.

Screenshots in `docs/images/` show only ScriptCraft with the bundled sample script, made with
`scriptcraft --sample --screenshot <file.png>`.

## See also

[README.md](README.md) · [docs/architecture.md](docs/architecture.md) ·
[docs/fountain.md](docs/fountain.md) · [docs/formatting.md](docs/formatting.md) ·
[docs/keyboard.md](docs/keyboard.md) · [docs/interop.md](docs/interop.md) ·
[docs/file-formats.md](docs/file-formats.md) · [docs/launcher-entry.toml](docs/launcher-entry.toml)
