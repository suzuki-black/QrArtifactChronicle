# Third-Party Notices

This project (MIT licensed) bundles or links the following third-party components.
All are compatible with MIT distribution. License audit: `cargo audit` (0 vulnerabilities),
licenses extracted via `cargo metadata`.

## License summary (Rust dependency tree)

| License | Count (approx.) | Commercial / MIT use |
|---------|-----------------|----------------------|
| MIT / Apache-2.0 / MIT OR Apache-2.0 | ~90 | ✅ permissive |
| BSD-2/3-Clause, Zlib, 0BSD, Unlicense | several | ✅ permissive (attribution) |
| Unicode-3.0 (`unicode-ident`) | 1 | ✅ permissive |
| **MPL-2.0** (UniFFI crates) | 8 | ⚠️ see below — compatible, with notice |

> Disjunctive licenses such as `r-efi` (`MIT OR Apache-2.0 OR LGPL-2.1-or-later`) are used
> under their **MIT/Apache** option. `r-efi` is a UEFI crate and is not compiled on
> macOS / iOS / Android targets.

## Notable components

- **UniFFI** (`uniffi`, `uniffi_*`) — © Mozilla, **MPL-2.0**.
  Used unmodified to generate the Swift/Kotlin FFI bindings (`qrac-ffi`).
  MPL-2.0 is file-level weak copyleft: combining it into a larger work (under MIT or a
  proprietary license) is permitted. Obligation when distributing binaries: keep the MPL
  notice and make the **source of the MPL files** available — they are published at
  https://github.com/mozilla/uniffi-rs . Your own source is not affected.
  *(Roadmap: replace UniFFI with a hand-written C ABI for a fully permissive tree — see README.)*
- **tiny-skia** (`tiny-skia`, `tiny-skia-path`) — **BSD-3-Clause**. CPU 2D rendering.
  Contains code derived from Skia (© Google, BSD-3-Clause).
- **SQLite** via `rusqlite` (`libsqlite3-sys`, `bundled` feature) — SQLite is **public domain**.
- **sha2 / serde / serde_json / unicode-normalization** — MIT OR Apache-2.0.

## Regenerating a full report

For a complete, line-by-line third-party license bundle (recommended before any binary
distribution), run e.g.:

```
cargo install cargo-about
cargo about generate about.hbs > THIRD_PARTY_FULL.html   # (from rust/)
```
