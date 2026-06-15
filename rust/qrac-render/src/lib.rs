//! qrac-render: I/O層（DB選択＋画像合成）。決定論ロジックは qrac-core に委譲。
pub mod art;
pub mod compose;
pub mod db;

pub use compose::{
    render, render_base_sprite, render_png, render_rgba, RgbaBuf, CANVAS_H, CANVAS_W,
};
pub use db::{ArtifactDb, BaseArtifact};
