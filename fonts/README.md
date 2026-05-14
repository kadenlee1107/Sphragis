# Embedded fonts

Two TrueType fonts are embedded into the kernel binary via `include_bytes!` for the boot/lock screen.

| File | Source | License | Use |
|------|--------|---------|-----|
| `ibm-plex-serif-italic.ttf` | [IBM/plex v6.4.2](https://github.com/IBM/plex) | SIL Open Font License 1.1 | Σ glyph on lock screen |
| `ibm-plex-sans-medium.ttf`  | [IBM/plex v6.4.2](https://github.com/IBM/plex) | SIL Open Font License 1.1 | "SPHRAGIS" wordmark |

Both files have been subsetted with `pyftsubset` (from `fonttools`) to the following codepoints to minimize kernel binary footprint:

- `U+0020` (space)
- `U+002D` (hyphen-minus)
- `U+002E` (period)
- `U+0030`–`U+0039` (digits 0–9)
- `U+0041`–`U+005A` (uppercase A–Z)
- `U+03A3` (Greek capital letter sigma — Σ)

The SIL OFL is compatible with this project's AGPL-3.0-or-later license. Per SIL OFL terms, the fonts may not be redistributed under a different name, which is why the filenames preserve `ibm-plex-` upstream branding.

## Legacy file

`font.ttf` is **Verdana** (Microsoft proprietary, bundled with macOS but not redistributable). It is left in place this wave because the rasterizer (`src/ui/truetype.rs`) historically referenced it. After Task 2 of the lock-screen-redesign implementation, nothing in the kernel still references it. Removal is a follow-up task tracked in `docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md` under *Scope boundary*.
