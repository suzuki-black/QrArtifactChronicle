//! ベース遺物DB（docs/03）。候補選択と6段フォールバック（docs/03 3.5 / docs/00 0.4.2）。
//! 選択は qrac_core::hash::uint_below(seed,"pick",len) で決定論。
//!
//! 注: スキーマは docs/03 3.2 を簡略化し civ/era/category を TEXT で持つ（id正規化テーブルは省略）。
//! 種データ（seed_full）は最小スタンドイン。本物のDB生成は qrac-assetgen（Step 3）が担う。

use qrac_core::constants::{CATEGORIES, CIVILIZATIONS, ERAS};
use qrac_core::hash::uint_below;
use rusqlite::{params, Connection, Params};

pub const TAG_PICK: &str = "pick";

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
                ON base_artifact(civ,era,category,base_rarity,id);",
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
}
