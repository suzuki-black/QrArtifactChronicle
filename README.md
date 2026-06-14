# QrArtifactChronicle 🏺

> Scan any QR code and excavate a fictional ancient artifact — deterministically.
> The same QR always yields the same artifact; a finite database feels infinite.

**🇯🇵 日本語版: [README.ja.md](README.ja.md)**

> ⚠️ **Prototype.** This is an early proof-of-concept. The deterministic core is solid
> and fully tested, but the app, artwork, and asset pipeline are placeholders meant for
> iteration. APIs, data formats, and visuals will change.

---

## What is this?

Read a real-world QR code → hash its contents → generate a fictional artifact of a made-up
ancient civilization. The artifact's rarity, civilization, era, category, colour, dirt,
damage and a tongue-in-cheek encyclopaedia blurb are all derived **deterministically** from
the QR. Collect them into an exhibition room.

- **Same QR → same artifact, always** (no randomness, no timestamps).
- **Finite base DB × infinite composition** — colour shifts, dirt/damage layers and flavour
  text make a few thousand base records feel limitless (>10¹² combinations).
- **Gentle gacha-style rarity** (★1 ≈ 40%, ★10 ≈ 0.01%).
- **Era bonus** — if the QR's content embeds a real date, older QRs reach the mythic ★11–★13.
- **Offline** — no LLM/server at runtime; everything is precomputed/bundled.

## How it works (determinism)

```
QR bytes → normalize → SHA-256 (seed) → tagged sub-streams → attributes
                                                  ├ rarity (integer thresholds, float-free)
                                                  ├ civilization / era / category
                                                  ├ colour (HSV), dirt, damage, preservation
                                                  └ era bonus (only if a real date is found)
        → DB select (civ × era × category × rarity, 6-stage fallback)
        → compose image (base sprite → HSV → dirt → damage → preservation → background)
        → 民明書房-style description
```

The whole pipeline is byte-for-byte reproducible. A TypeScript reference implementation acts
as a language-neutral **oracle**; the Rust core must reproduce its golden vectors exactly:

```
TS reference  ≡  vectors/golden.json  ≡  Rust qrac-core  ≡  Swift (via FFI)
```

## Repository layout

```
docs/                 Design specification (00–08) + open-questions log
reference/            TypeScript reference core (the oracle) + golden vectors + tests
rust/
  qrac-core/          Pure deterministic core (hash, normalize, rarity, timestamp,
                      appearance, flavor, derive) — verified against golden.json
  qrac-render/        I/O layer: SQLite base-artifact DB (rusqlite) + image compose (tiny-skia)
  qrac-ffi/           UniFFI boundary exposing the core to Swift/Kotlin
  qrac-assetgen/      Asset generator: base images + full-coverage DB + CI coverage gate
apple/                macOS SwiftUI app (QrArtifactChronicle.app) + XCFramework packaging
```

## Build & run

**Rust core tests** (determinism, distribution, independence, DB fallback, compose):
```bash
cd rust && cargo test --workspace
```

**TypeScript oracle** (regenerate / verify golden vectors):
```bash
cd reference && npm test
```

**Generate assets** (base images + `artifacts.sqlite`, with coverage gate):
```bash
cd rust && cargo run -p qrac-assetgen -- dist
```

**macOS app** (builds the Rust core, packages an XCFramework, assembles the .app):
```bash
cd apple && bash build-app.sh
open QrArtifactChronicle.app    # unsigned: first launch may need right-click → Open
```

Requirements: Rust (stable), Node ≥ 23.6 (for the reference's native TS execution),
Xcode / Swift 6 toolchain (for the macOS app).

## 📷 Camera scanning notes

The app scans QR codes live (Vision `VNDetectBarcodesRequest`, no shutter, no auto-opening
of links). **The MacBook built-in camera is fixed-focus and often struggles with QR codes.**

If scanning fails:
- **Use your iPhone as a Continuity Camera** — its autofocus reads QR codes easily. Place the
  iPhone near the Mac (same Apple ID, Wi-Fi + Bluetooth on), keep it still & locked, then pick
  it from the camera menu in the scan screen (use *Re-scan cameras* if it isn't listed yet).
- Or an external USB webcam.
- For the built-in camera: hold the QR ~30–50 cm away, make it large and bright, avoid glare.

## Roadmap / TODO

- [ ] **Replace UniFFI (MPL-2.0) with a hand-written C ABI FFI** for a fully permissive
      dependency tree (only needed for a strict closed-source/commercial posture; MPL is fine
      for the current MIT/OSS release).
- [ ] iOS & Android targets (multi-platform XCFramework; Kotlin bindings via cargo-ndk).
- [ ] Replace procedural placeholder art with real artist assets (WebP). Asset layout/naming
      is already swap-ready.
- [ ] Runtime 6-layer compositing with real dirt/damage PNG layers + blend modes (currently a
      procedural stand-in).
- [ ] Lightweight collection model (regenerate from key) instead of storing rendered PNGs.
- [ ] Code signing & notarization for distribution.

## License

**MIT © 2026 suzuki-black** — see [LICENSE](LICENSE).

Third-party components (all MIT-compatible; UniFFI is MPL-2.0 and used unmodified) are listed
in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

> The "民明書房" (Minmei Shobō) flavour-text style is an homage to *Sakigake!! Otokojuku*;
> the publisher name and all blurbs in this project are original, generated text.
