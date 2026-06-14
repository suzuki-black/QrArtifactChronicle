# Contributing

Thanks for your interest! This is an early **prototype** — issues and PRs welcome.

## Development

```bash
# Rust deterministic core
cd rust && cargo test --workspace        # tests
cargo fmt && cargo clippy --workspace    # format + lint (CI enforces both)

# TypeScript reference oracle
cd reference && npm test                 # Node >= 23.6

# Assets + macOS app
cd rust && cargo run -p qrac-assetgen -- dist
cd apple && bash build-app.sh && open QrArtifactChronicle.app
```

## ⚠️ Determinism is the core invariant

"The same QR always yields the same artifact." If you change anything that affects the
hash → attribute derivation, you change everyone's artifacts. So:

- **`rust/qrac-core` must stay byte-identical to `reference/` (TypeScript).** Both are checked
  against `reference/vectors/golden.json`.
- Changing RNG tags, `RARITY_CUTOFFS_U32`/`DAMAGE_CUTOFF_*`, weights, the URL normalization
  rules, the layer/compose order, or `artifactHash` derivation is a **breaking change**:
  bump `GEN_VERSION` (`constants.rs`) and regenerate golden vectors
  (`cd reference && npm run gen:vectors`), then make the Rust `golden.rs` test pass.
- Discrete decisions use **integer thresholds** (no floats); only continuous outputs
  (colour, preservation) use `f64`. See `docs/08` §8.4.

## Where things live

See `docs/` (design 00–08) and the per-crate READMEs. `qrac-core` is pure; `qrac-render`
is I/O (DB + image compose); `qrac-ffi` is the UniFFI boundary; `qrac-assetgen` builds assets.
