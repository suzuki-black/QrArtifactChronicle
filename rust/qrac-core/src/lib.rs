//! qrac-core: QrArtifactChronicle 決定論コア（Rust）。
//! 純粋部のみ（normalize/hash/rarity/timestamp/appearance/derive）。
//! reference/(TS) と関数単位で対応し、golden vectors で byte一致を検証（docs/08 8.9）。

pub mod appearance;
pub mod constants;
pub mod derive;
pub mod flavor;
pub mod genealogy;
pub mod hash;
pub mod normalize;
pub mod rarity;
pub mod timestamp;
pub mod types;

pub use derive::{derive_attributes, derive_from_string};
pub use flavor::Lang;
pub use genealogy::{
    category_label, civ_label, decode_image_set, describe_pair, encode_image_set, era_label,
    type_label, type_label_from_set, RefKind, RefMeta,
};
pub use types::{ColorMod, Damage, DerivedAttributes};
