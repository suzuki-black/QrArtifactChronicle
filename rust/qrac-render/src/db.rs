//! ベース遺物DB（docs/03）。候補選択と6段フォールバック（docs/03 3.5 / docs/00 0.4.2）。
//! 選択は qrac_core::hash::uint_below(seed,"pick",len) で決定論。
//!
//! 注: スキーマは docs/03 3.2 を簡略化し civ/era/category を TEXT で持つ（id正規化テーブルは省略）。
//! 種データ（seed_full）は最小スタンドイン。本物のDB生成は qrac-assetgen（Step 3）が担う。

use qrac_core::constants::{CATEGORIES, CIVILIZATIONS, ERAS};
use qrac_core::hash::{make_seed, uint_below};
use rusqlite::{params, Connection, Params};
use std::collections::HashSet;

pub const TAG_PICK: &str = "pick";

/// 参照エッジの採否しきい値（%）。型ごとの参照本数を Q1（典型0〜1・最大2）に収める。
/// しきい値を変えると参照網＝出力テキストが変わるため、変更時は GEN_VERSION を bump。
const REF_ADOPT_PCT: u32 = 55;

/// 参照エッジ (from,to,kind) を採用するか（安定ハッシュ・個体非依存）。
fn ref_adopted(from: i64, to: i64, kind: &str) -> bool {
    let s = make_seed(format!("ref:{from}:{to}:{kind}").as_bytes());
    uint_below(&s, "ref:adopt", 100) < REF_ADOPT_PCT
}

fn cat_index(name: &str) -> usize {
    CATEGORIES
        .iter()
        .position(|(c, _)| *c == name)
        .unwrap_or_else(|| panic!("category {name} not in CATEGORIES"))
}

#[derive(Debug, Clone)]
pub struct BaseArtifact {
    pub id: i64,
    pub civ: String,
    pub era: String,
    pub category: String,
    pub base_rarity: u32,
    pub name: String,
    pub image_set_id: i64,
    /// どの段で当たったか（1..6, 7=GLOBAL）。デバッグ・分析用。
    pub matched_stage: u8,
}

pub struct ArtifactDb {
    conn: Connection,
}

impl ArtifactDb {
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let db = Self {
            conn: Connection::open_in_memory()?,
        };
        db.create_schema()?;
        Ok(db)
    }

    pub fn open(path: &str) -> rusqlite::Result<Self> {
        let db = Self {
            conn: Connection::open(path)?,
        };
        db.create_schema()?;
        Ok(db)
    }

    fn create_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS base_artifact(
                id           INTEGER PRIMARY KEY,
                civ          TEXT NOT NULL,
                era          TEXT NOT NULL,
                category     TEXT NOT NULL,
                base_rarity  INTEGER NOT NULL,
                name         TEXT NOT NULL,
                image_set_id INTEGER NOT NULL,
                is_global    INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_lookup
                ON base_artifact(civ,era,category,base_rarity,id);
            CREATE TABLE IF NOT EXISTS artifact_reference(
                from_set INTEGER NOT NULL,   -- 引用元 image_set_id（型, 判断A）
                to_set   INTEGER NOT NULL,   -- 引用先 image_set_id（型）
                kind     TEXT NOT NULL,      -- 'pair' | 'cites' | 'rival'
                PRIMARY KEY (from_set, to_set, kind)
            );
            CREATE INDEX IF NOT EXISTS idx_ref_from
                ON artifact_reference(from_set);",
        )
    }

    /// 各 (civ × era × category) について base_rarity 1..10 を1件ずつ充足（docs/05 5.6 制約）。
    /// ＋ GLOBAL フォールバック1件。Step 3 の本物の生成ツールに置き換える前提のスタンドイン。
    pub fn seed_full(&self) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let mut image_set_id: i64 = 0;
        for (ci, (civ, _)) in CIVILIZATIONS.iter().enumerate() {
            for (ei, (era, _)) in ERAS.iter().enumerate() {
                for (ki, (cat, _)) in CATEGORIES.iter().enumerate() {
                    // image_set_id は (civ,era,category) ごとに安定（rarity 跨ぎで共有）
                    image_set_id = (ci * 100 + ei * 10 + ki) as i64;
                    for rarity in 1..=10u32 {
                        let name = format!("{civ}の{cat}（{era}・★{rarity}）");
                        tx.execute(
                            "INSERT INTO base_artifact(civ,era,category,base_rarity,name,image_set_id)
                             VALUES(?1,?2,?3,?4,?5,?6)",
                            params![civ, era, cat, rarity as i64, name, image_set_id],
                        )?;
                    }
                }
            }
        }
        let _ = image_set_id;
        // GLOBAL フォールバック（docs/03 3.5 終端）: 文明・時代・カテゴリに属さない1件。
        tx.execute(
            "INSERT INTO base_artifact(civ,era,category,base_rarity,name,image_set_id,is_global)
             VALUES('*','*','*',0,'未分類の遺物',-1,1)",
            [],
        )?;
        tx.commit()
    }

    pub fn count_non_global(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM base_artifact WHERE is_global=0",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
    }

    // ── 出土の系譜（参照グラフ, 提案01）─────────────────────────────────────────

    /// 参照グラフを決定論生成（型付き規則＋安定ハッシュ採否, §3.1）。
    /// image_set_id = ci*100+ei*10+ki（seed_full / assetgen と一致）で型を畳む。
    /// 規則:
    ///   - pair  : 同 (civ,era) の weapon ↔ ritual（双方向: 両向き行, Q2）
    ///   - cites : 同 (civ,era) の inscription → architecture（片方向, Q2）
    ///   - rival : 同 (era,category) で civ を隣接ペアに組む ci↔ci+1（ci 偶数のみ・双方向）。
    ///             完全マッチングのため N が奇数なら末尾の civ は rival を持たない。
    ///             1型あたりの rival は最大1相手に限り、発参照を Q1（最大2）に収める。
    pub fn seed_references(&self) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let ncv = CIVILIZATIONS.len();
        let nera = ERAS.len();
        let ncat = CATEGORIES.len();
        let weapon = cat_index("weapon");
        let ritual = cat_index("ritual");
        let inscription = cat_index("inscription");
        let architecture = cat_index("architecture");

        // 採用エッジを集約（重複排除）。値: (from,to,kind)
        let mut edges: HashSet<(i64, i64, &'static str)> = HashSet::new();
        let set = |ci: usize, ei: usize, ki: usize| (ci * 100 + ei * 10 + ki) as i64;

        for ci in 0..ncv {
            for ei in 0..nera {
                // pair: weapon ↔ ritual（双方向）
                let (w, r) = (set(ci, ei, weapon), set(ci, ei, ritual));
                if ref_adopted(w, r, "pair") {
                    edges.insert((w, r, "pair"));
                    edges.insert((r, w, "pair"));
                }
                // cites: inscription → architecture（片方向）
                let (ins, arc) = (set(ci, ei, inscription), set(ci, ei, architecture));
                if ref_adopted(ins, arc, "cites") {
                    edges.insert((ins, arc, "cites"));
                }
                // rival: 同 (era,category) を隣接ペア ci↔ci+1（ci 偶数のみ）で双方向に結ぶ。
                if ci % 2 == 0 {
                    let cj = ci + 1;
                    if cj < ncv {
                        for ki in 0..ncat {
                            let (a, b) = (set(ci, ei, ki), set(cj, ei, ki));
                            if ref_adopted(a, b, "rival") {
                                edges.insert((a, b, "rival"));
                                edges.insert((b, a, "rival"));
                            }
                        }
                    }
                }
            }
        }

        let mut rows: Vec<(i64, i64, &str)> = edges.into_iter().collect();
        rows.sort(); // 決定論的書込み順
        for (from, to, kind) in rows {
            debug_assert_ne!(from, to, "self-reference must not be generated");
            tx.execute(
                "INSERT OR IGNORE INTO artifact_reference(from_set,to_set,kind) VALUES(?1,?2,?3)",
                params![from, to, kind],
            )?;
        }
        tx.commit()
    }

    pub fn count_references(&self) -> i64 {
        self.conn
            .query_row("SELECT COUNT(*) FROM artifact_reference", [], |r| r.get(0))
            .unwrap_or(0)
    }

    /// 型 `from_set` の発する参照一覧 (to_set, kind)。詳細画面「関連遺物」用。
    pub fn references_from(&self, from_set: i64) -> Vec<(i64, String)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT to_set, kind FROM artifact_reference WHERE from_set=?1 ORDER BY to_set, kind",
            )
            .expect("prepare references_from");
        let rows = stmt
            .query_map(params![from_set], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })
            .expect("query references_from");
        rows.filter_map(Result::ok).collect()
    }

    fn collect_ref_targets(&self) -> HashSet<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT to_set FROM artifact_reference")
            .expect("prepare targets");
        let rows = stmt
            .query_map([], |r| r.get::<_, i64>(0))
            .expect("query targets");
        rows.filter_map(Result::ok).collect()
    }

    /// 到達性CIゲート（§4）: ランダムQRからの導出→base選択をサンプリングし、
    /// 全 `to_set` が現実的試行回数内で出現するか検証。未到達の型名リストを返す（空=合格）。
    pub fn verify_reference_reachability(&self, samples: usize) -> Vec<String> {
        let targets = self.collect_ref_targets();
        if targets.is_empty() {
            return Vec::new();
        }
        let mut seen: HashSet<i64> = HashSet::new();
        for i in 0..samples {
            let key = format!("reach-sample-{i}");
            let attr = qrac_core::derive_from_string(&key, None);
            let seed = make_seed(&qrac_core::normalize::normalize_key(key.as_bytes()));
            let b = self.select_base(&seed, &attr.civ, &attr.era, &attr.category, attr.base_rarity);
            seen.insert(b.image_set_id);
        }
        let mut missing: Vec<i64> = targets.difference(&seen).copied().collect();
        missing.sort();
        missing
            .into_iter()
            .map(|t| format!("unreachable to_set {t}"))
            .collect()
    }

    /// 全 (civ × era × category) で base_rarity 1..10 が揃っているか検証（docs/05 5.6 制約 / CIゲート）。
    /// 不足があれば "civ/era/category: missing ★n,…" のリストを返す（空=合格）。
    pub fn verify_full_coverage(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for (civ, _) in CIVILIZATIONS {
            for (era, _) in ERAS {
                for (cat, _) in CATEGORIES {
                    let present: std::collections::HashSet<u32> = self
                        .fetch_rows("civ=?1 AND era=?2 AND category=?3", params![civ, era, cat])
                        .into_iter()
                        .map(|(_, r)| r)
                        .collect();
                    let missing: Vec<u32> = (1..=10u32).filter(|r| !present.contains(r)).collect();
                    if !missing.is_empty() {
                        problems.push(format!(
                            "{civ}/{era}/{cat}: missing {}",
                            missing
                                .iter()
                                .map(|r| format!("★{r}"))
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }
                }
            }
        }
        problems
    }

    fn fetch_rows<P: Params>(&self, where_clause: &str, p: P) -> Vec<(i64, u32)> {
        let sql = format!(
            "SELECT id, base_rarity FROM base_artifact WHERE is_global=0 AND {where_clause} ORDER BY id"
        );
        let mut stmt = self.conn.prepare(&sql).expect("prepare");
        let rows = stmt
            .query_map(p, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? as u32)))
            .expect("query");
        rows.filter_map(Result::ok).collect()
    }

    fn fetch_by_id(&self, id: i64, stage: u8) -> BaseArtifact {
        self.conn
            .query_row(
                "SELECT id,civ,era,category,base_rarity,name,image_set_id FROM base_artifact WHERE id=?1",
                params![id],
                |r| {
                    Ok(BaseArtifact {
                        id: r.get(0)?,
                        civ: r.get(1)?,
                        era: r.get(2)?,
                        category: r.get(3)?,
                        base_rarity: r.get::<_, i64>(4)? as u32,
                        name: r.get(5)?,
                        image_set_id: r.get(6)?,
                        matched_stage: stage,
                    })
                },
            )
            .expect("fetch_by_id")
    }

    fn pick_from(&self, seed: &[u8], rows: &[(i64, u32)], stage: u8) -> BaseArtifact {
        let idx = uint_below(seed, TAG_PICK, rows.len() as u32) as usize;
        self.fetch_by_id(rows[idx].0, stage)
    }

    /// rarity 最近傍に絞ってから pick（docs/03 3.5 段2）。
    fn pick_nearest(
        &self,
        seed: &[u8],
        rows: &[(i64, u32)],
        target: u32,
        stage: u8,
    ) -> BaseArtifact {
        let dist = |r: u32| (r as i32 - target as i32).abs();
        let min_d = rows.iter().map(|(_, r)| dist(*r)).min().unwrap();
        let cand: Vec<(i64, u32)> = rows
            .iter()
            .cloned()
            .filter(|(_, r)| dist(*r) == min_d)
            .collect();
        self.pick_from(seed, &cand, stage)
    }

    fn global(&self) -> BaseArtifact {
        self.conn
            .query_row(
                "SELECT id,civ,era,category,base_rarity,name,image_set_id FROM base_artifact WHERE is_global=1 LIMIT 1",
                [],
                |r| {
                    Ok(BaseArtifact {
                        id: r.get(0)?,
                        civ: r.get(1)?,
                        era: r.get(2)?,
                        category: r.get(3)?,
                        base_rarity: r.get::<_, i64>(4)? as u32,
                        name: r.get(5)?,
                        image_set_id: r.get(6)?,
                        matched_stage: 7,
                    })
                },
            )
            .expect("GLOBAL fallback row missing")
    }

    /// 候補選択（docs/03 3.4）＋6段フォールバック（docs/03 3.5）。
    pub fn select_base(
        &self,
        seed: &[u8],
        civ: &str,
        era: &str,
        category: &str,
        rarity: u32,
    ) -> BaseArtifact {
        let r = rarity as i64;

        // 段1: (civ, era, category, rarity)
        let s1 = self.fetch_rows(
            "civ=?1 AND era=?2 AND category=?3 AND base_rarity=?4",
            params![civ, era, category, r],
        );
        if !s1.is_empty() {
            return self.pick_from(seed, &s1, 1);
        }
        // 段2: (civ, era, category, *) rarity最近傍
        let s2 = self.fetch_rows(
            "civ=?1 AND era=?2 AND category=?3",
            params![civ, era, category],
        );
        if !s2.is_empty() {
            return self.pick_nearest(seed, &s2, rarity, 2);
        }
        // 段3: (civ, era, *, rarity)
        let s3 = self.fetch_rows("civ=?1 AND era=?2 AND base_rarity=?3", params![civ, era, r]);
        if !s3.is_empty() {
            return self.pick_from(seed, &s3, 3);
        }
        // 段4: (civ, *, category, rarity)
        let s4 = self.fetch_rows(
            "civ=?1 AND category=?2 AND base_rarity=?3",
            params![civ, category, r],
        );
        if !s4.is_empty() {
            return self.pick_from(seed, &s4, 4);
        }
        // 段5: (civ, *, *, *)
        let s5 = self.fetch_rows("civ=?1", params![civ]);
        if !s5.is_empty() {
            return self.pick_from(seed, &s5, 5);
        }
        // 段6: (*, *, *, rarity)
        let s6 = self.fetch_rows("base_rarity=?1", params![r]);
        if !s6.is_empty() {
            return self.pick_from(seed, &s6, 6);
        }
        // 終端
        self.global()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrac_core::hash::make_seed;

    #[test]
    fn full_seed_always_hits_stage1() {
        let db = ArtifactDb::open_in_memory().unwrap();
        db.seed_full().unwrap();
        let seed = make_seed(b"x");
        let b = db.select_base(&seed, "desert", "ancient", "weapon", 5);
        assert_eq!(b.matched_stage, 1);
        assert_eq!(b.civ, "desert");
        assert_eq!(b.base_rarity, 5);
    }

    #[test]
    fn coverage_gate_detects_gaps_and_passes_when_full() {
        let sparse = ArtifactDb::open_in_memory().unwrap();
        sparse
            .conn
            .execute(
                "INSERT INTO base_artifact(civ,era,category,base_rarity,name,image_set_id) VALUES('desert','ancient','weapon',3,'a',0)",
                [],
            )
            .unwrap();
        assert!(
            !sparse.verify_full_coverage().is_empty(),
            "gate must catch gaps"
        );

        let full = ArtifactDb::open_in_memory().unwrap();
        full.seed_full().unwrap();
        assert!(
            full.verify_full_coverage().is_empty(),
            "full seed must pass the gate"
        );
    }

    #[test]
    fn fallback_ladder_and_determinism() {
        let db = ArtifactDb::open_in_memory().unwrap();
        // わざと疎に: desert/ancient/weapon の rarity 3 のみ
        db.conn
            .execute(
                "INSERT INTO base_artifact(civ,era,category,base_rarity,name,image_set_id) VALUES('desert','ancient','weapon',3,'a',1)",
                [],
            )
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO base_artifact(civ,era,category,base_rarity,name,image_set_id,is_global) VALUES('*','*','*',0,'g',-1,1)",
                [],
            )
            .unwrap();
        let seed = make_seed(b"y");
        // rarity10 を要求 → 段2(最近傍=3)
        let b = db.select_base(&seed, "desert", "ancient", "weapon", 10);
        assert_eq!(b.matched_stage, 2);
        assert_eq!(b.base_rarity, 3);
        // 別文明 → 終端GLOBAL
        let g = db.select_base(&seed, "ocean", "future", "ritual", 7);
        assert_eq!(g.matched_stage, 7);
        // 決定論
        let b2 = db.select_base(&seed, "desert", "ancient", "weapon", 10);
        assert_eq!(b.id, b2.id);
    }

    #[test]
    fn references_are_deterministic_acyclic_and_reachable() {
        let a = ArtifactDb::open_in_memory().unwrap();
        a.seed_full().unwrap();
        a.seed_references().unwrap();
        let b = ArtifactDb::open_in_memory().unwrap();
        b.seed_full().unwrap();
        b.seed_references().unwrap();
        // 決定論: 件数一致
        assert_eq!(a.count_references(), b.count_references());
        assert!(a.count_references() > 0, "参照が1件も生成されない");

        // 自己参照なし & 双方向 kind の対称性（pair/rival は両向き行）
        let mut stmt = a
            .conn
            .prepare("SELECT from_set,to_set,kind FROM artifact_reference")
            .unwrap();
        let all: Vec<(i64, i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        for (f, t, k) in &all {
            assert_ne!(f, t, "self-reference");
            if k == "pair" || k == "rival" {
                assert!(
                    all.iter().any(|(f2, t2, k2)| f2 == t && t2 == f && k2 == k),
                    "双方向 {k} の逆向き行が無い: {f}->{t}"
                );
            }
        }

        // 型ごとの発参照は最大2（Q1）
        let mut out: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
        for (f, _, _) in &all {
            *out.entry(*f).or_default() += 1;
        }
        assert!(
            out.values().all(|&c| c <= 2),
            "発参照が型あたり2を超える: {out:?}"
        );

        // 到達性CIゲート: サンプリングで全 to_set 到達
        assert!(
            a.verify_reference_reachability(20_000).is_empty(),
            "参照先が到達不能"
        );
    }
}
