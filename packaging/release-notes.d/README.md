# Release-note fragments

A pull request with a user-visible change adds one file here,
`<issue>-<branch-slug>.md`, holding its release-note entry exactly as it reads: one `- ` bullet,
written for users. The release workflow joins the fragments into the release notes and deletes
them. Pull requests never edit a shared release-notes file, so they never conflict on it.
`cargo xtask progress` (part of `cargo xtask ci`) checks the shape. Same scheme as the progress
log in `docs/progress.d/` (craft-contrib RFC-0017).
