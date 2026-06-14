//! 遺物の解説文（民明書房調）。docs/04 4.5 の個体差テキストの一種。
//! 「詳説 世界の遺物（萬象書房 1890年刊）」からの抜粋という体裁。決定論（seed 由来）。
use crate::hash::uint_below;
use crate::types::DerivedAttributes;

fn pick<'a>(seed: &[u8], tag: &str, arr: &[&'a str]) -> &'a str {
    arr[uint_below(seed, tag, arr.len() as u32) as usize]
}

/// 架空の発掘地名（決定論）。
fn gen_place(seed: &[u8]) -> String {
    const SYL: [&str; 14] = [
        "ア", "ル", "サ", "ネブ", "カ", "トゥ", "メ", "ゾ", "ラ", "ウ", "ヴ", "ガ", "シ", "ロ",
    ];
    const SUF: [&str; 6] = ["盆地", "遺跡", "谷", "高原", "砂海", "氷河"];
    let n = 2 + uint_below(seed, "text:place.len", 2); // 2..3 音節
    let mut name = String::new();
    for i in 0..n {
        let idx = uint_below(seed, &format!("text:place.{i}"), SYL.len() as u32) as usize;
        name.push_str(SYL[idx]);
    }
    let suf = SUF[uint_below(seed, "text:place.suf", SUF.len() as u32) as usize];
    format!("“{name}{suf}”")
}

/// 民明書房調の解説文を生成する。
pub fn describe(seed: &[u8], a: &DerivedAttributes) -> String {
    let civ = match a.civ.as_str() {
        "desert" => "灼熱の砂漠に栄えし砂上の王朝",
        "ocean" => "大洋の底に沈みし海洋文明",
        "mountain" => "万年雪の霊峰に隠れ住みし山岳の民",
        "machine" => "歯車仕掛けの神を奉りし古代機械文明",
        "organic" => "生命と渾然一体たりし有機文明",
        _ => "いずことも知れぬ忘れられし民",
    };
    let era = match a.era.as_str() {
        "ancient" => "太古",
        "medieval" => "中世",
        "early_modern" => "近世",
        "modern" => "近代",
        "future" => "未だ来たらざる未来",
        _ => "悠久の刻",
    };
    let cat = match a.category.as_str() {
        "weapon" => "神聖なる武具",
        "ritual" => "厳粛なる祭祀の具",
        "daily" => "日々の暮らしを支えし什器",
        "architecture" => "壮麗なる殿堂の一片",
        "inscription" => "秘奥の文を刻みし碑",
        _ => "精緻を極めし機巧の部品",
    };

    let place = gen_place(seed);
    let opening = format!("{place}より出土せし本品は、{civ}に興りし、{era}の{cat}に他ならぬ。");

    let etym = pick(
        seed,
        "text:desc.etym",
        &[
            "今日、万人が時を計るに用うる暦法の根本が、実にこの文様より生じたことは、もはや論を俟たぬ。",
            "かの古の大賢が天地創成の理を悟りしは、これを一目せし刹那であったと、古文書は確かに伝えている。",
            "東西の交易路を遥々と渡り、遠き異邦の王侯すらも虜にしたというのだから、その価値は推して知るべしである。",
            "我々が日々何気なく口にする数多の言の葉、その語源のことごとくが本品に発すると説く碩学も、決して少なくはない。",
            "世界各地に散らばる『失われし宝』の伝説、その真の出所がまさしくこれであるとは、知る人ぞ知る秘事である。",
            "後世これを模したる贋作が無数に作られたが、いずれも本品の神韻には遠く及ばなんだと、記録は一様に嘆いている。",
        ],
    );

    let surface = match a.dirt_layer_id.as_str() {
        "volcanic_ash" => "表面を黒々と覆う層は、かつて天を覆い尽くせし大噴火の灰に相違あるまい。",
        "mud" => {
            "こびり付きたる泥土は、これが聖なる大河の底にて千年の眠りに就きし何よりの証左である。"
        }
        "sand" => "肌に噛みたる細砂は、無数の隊商がこれを求めて砂漠を彷徨いし執念の名残であろう。",
        "soot" => "煤けたる肌は、夜毎に絶ゆることなく焚かれし祭祀の炎を、今に静かに伝えている。",
        "sea_salt" => "白く吹きたる塩の結晶は、七つの海を越え来たりし数奇なる運命を雄弁に物語る。",
        _ => {
            if a.base_rarity >= 7 {
                "幾星霜を閲してなお当時の輝きを一片たりとも失わざるは、まさに奇跡という他に言葉を知らぬ。"
            } else {
                "風雪に晒されし肌の侘びこそ、かえって本品の真贋を疑う余地なきものとしている。"
            }
        }
    };

    let closing = pick(
        seed,
        "text:desc.close",
        &[
            "——この事実を知る者、今や天が下にただ一人とてあるまい。",
            "……諸君、これこそが歴史の闇に永く葬られし真実である。",
            "本書房が多年の研鑽の末、ここに白日の下へと晒すものなり。",
            "知らぬを以て恥とせば、まずは襟を正して本書を繙くがよかろう。",
        ],
    );

    let prefix = if a.is_mythic {
        "世に名高き神話級の至宝――。\n"
    } else {
        ""
    };
    format!("{prefix}{opening}{etym}{surface}{closing}")
}
