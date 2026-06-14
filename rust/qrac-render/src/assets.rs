//! アセット読込（docs/05 5.5 レイアウト）。compose は I/O を持たないので、
//! ファイル読込はここに分離する。

use std::path::Path;

/// `<assets_dir>/base/<image_set_id>/<style>/0.png` を読む。無ければ None（→手続きフォールバック）。
pub fn load_base_png(assets_dir: &Path, image_set_id: i64, style: &str) -> Option<Vec<u8>> {
    let p = assets_dir
        .join("base")
        .join(image_set_id.to_string())
        .join(style)
        .join("0.png");
    std::fs::read(p).ok()
}
