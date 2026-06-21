//! 出土の系譜（提案01）の決定論＋ゴールデンスナップショット（判断B: TS≡Rustパリティは課さない）。
//! 代表入力→期待テキストを固定し、生成ロジック変更時の意図せぬ差分を検出する。
//! テキストを意図的に変えた場合は本スナップショットと GEN_VERSION を併せて更新すること。

use qrac_core::flavor::describe;
use qrac_core::hash::make_seed;
use qrac_core::normalize::normalize_key;
use qrac_core::{
    describe_pair, derive_from_string, encode_image_set, type_label, type_label_from_set, Lang,
    RefKind, RefMeta,
};

fn seed_for(text: &str) -> [u8; 32] {
    make_seed(&normalize_key(text.as_bytes()))
}

#[test]
fn type_label_snapshots() {
    // (civ, era, category, lang) → 期待呼称。詳細チップと同語彙。
    assert_eq!(
        type_label("ocean", "ancient", "ritual", Lang::Ja),
        "海洋文明・古代の祭具"
    );
    assert_eq!(
        type_label("ocean", "ancient", "ritual", Lang::En),
        "Ritual of the Ancient Ocean civilization"
    );
    assert_eq!(
        type_label("desert", "future", "weapon", Lang::Ja),
        "砂漠文明・未来の武器"
    );
    assert_eq!(
        type_label("machine", "modern", "machine_part", Lang::En),
        "Machine part of the Modern Machine civilization"
    );
    // image_set_id からの解決も一致（ocean=1, ancient=0, ritual=1 → 101）
    assert_eq!(
        type_label_from_set(encode_image_set(1, 0, 1), Lang::Ja),
        "海洋文明・古代の祭具"
    );
}

#[test]
fn describe_pair_snapshots_and_canonical() {
    let a = encode_image_set(0, 0, 0); // 砂漠・太古・武具
    let b = encode_image_set(0, 0, 1); // 砂漠・太古・祭祀具
    let ja = describe_pair(a, b, RefKind::Pair, Lang::Ja);
    // 順不同で同一（canonical, 判断E）
    assert_eq!(ja, describe_pair(b, a, RefKind::Pair, Lang::Ja));
    // 両型呼称を含み、合本の体裁である
    assert!(ja.contains("砂漠文明・古代の武器"), "{ja}");
    assert!(ja.contains("砂漠文明・古代の祭具"), "{ja}");
    assert!(ja.starts_with("〈"), "{ja}");

    let en = describe_pair(a, b, RefKind::Rival, Lang::En);
    assert_eq!(en, describe_pair(b, a, RefKind::Rival, Lang::En));
    assert!(en.contains("Weapon of the Ancient Desert civilization"), "{en}");
}

#[test]
fn describe_without_refs_is_legacy() {
    let attr = derive_from_string("https://example.com/relic", None);
    let seed = seed_for("https://example.com/relic");
    let plain = describe(&seed, &attr, &[], Lang::Ja);
    // 参照文の接頭辞を含まない（refs 空 → 従来どおり）
    assert!(!plain.contains("なお本品は、"));
    assert!(!plain.contains("その銘の片隅には"));
    assert!(!plain.contains("また〈"));
    // 決定論
    assert_eq!(plain, describe(&seed, &attr, &[], Lang::Ja));
}

#[test]
fn describe_with_single_ref_appends_expected_sentence() {
    let attr = derive_from_string("genealogy-fixture-001", None);
    let seed = seed_for("genealogy-fixture-001");
    let label = "大洋文明・中世の碑文".to_string();
    let refs = vec![RefMeta {
        kind: RefKind::Pair,
        to_set: encode_image_set(1, 1, 4),
        to_label: label.clone(),
    }];
    let with_ref = describe(&seed, &attr, &refs, Lang::Ja);
    let plain = describe(&seed, &attr, &[], Lang::Ja);
    // 単一 ref（idx=0 確定）→ pair テンプレが本文末尾に正確に付加される
    let expected_tail = format!("なお本品は、〈{label}〉と対をなすものと伝わる。");
    assert_eq!(with_ref, format!("{plain}{expected_tail}"));

    // EN も同様（先頭スペース連結）
    let with_ref_en = describe(&seed, &attr, &refs, Lang::En);
    assert!(with_ref_en.contains(&format!(" It is, moreover, said to form a pair with the 〈{label}〉.")));
}

#[test]
fn describe_ref_selection_is_deterministic() {
    let attr = derive_from_string("multi-ref-fixture", None);
    let seed = seed_for("multi-ref-fixture");
    let refs = vec![
        RefMeta { kind: RefKind::Pair, to_set: 10, to_label: "A".into() },
        RefMeta { kind: RefKind::Cites, to_set: 20, to_label: "B".into() },
        RefMeta { kind: RefKind::Rival, to_set: 30, to_label: "C".into() },
    ];
    let a = describe(&seed, &attr, &refs, Lang::Ja);
    let b = describe(&seed, &attr, &refs, Lang::Ja);
    assert_eq!(a, b, "参照選択は決定論");
}
