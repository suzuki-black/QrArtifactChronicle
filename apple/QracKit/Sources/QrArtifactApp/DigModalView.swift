import SwiftUI

/// ファミコン風パラパラアニメのモーダル。約3秒後に onDone。
/// scolding=false: 女の子がツルハシで採掘 / true: 女の先生が💢プンプン。
struct DigModalView: View {
    let scolding: Bool
    let onDone: () -> Void
    private let step = 0.16   // パラパラのコマ送り（≈6fps）
    @State private var showBanner = false

    var body: some View {
        ZStack {
            Color.black.opacity(0.82).ignoresSafeArea()

            VStack(spacing: 14) {
                if showBanner {
                    banner
                } else {
                    animation
                }
            }
            .padding(22)
            .frame(width: 280)
            .background(Color(red: 0.07, green: 0.07, blue: 0.12))
            .overlay(Rectangle().stroke(.white, lineWidth: 4))
            .overlay(Rectangle().inset(by: -6).stroke(.white.opacity(0.5), lineWidth: 2))
            .padding(24)
        }
        .task {
            // アニメ中: 採掘=ざっくざっく / 既出=ぴーぴー
            SoundPlayer.shared.play(scolding ? "scold" : "dig")
            try? await Task.sleep(nanoseconds: 2_200_000_000)   // アニメ ~2.2s
            SoundPlayer.shared.stop(scolding ? "scold" : "dig")
            withAnimation(.easeIn(duration: 0.1)) { showBanner = true }
            // バナー: 新発見=てれって〜 / 既出=でーん
            SoundPlayer.shared.play(scolding ? "error" : "reveal")
            try? await Task.sleep(nanoseconds: 1_100_000_000)   // バナー ~1.1s
            onDone()
        }
    }

    private var animation: some View {
        VStack(spacing: 14) {
            TimelineView(.periodic(from: .now, by: step)) { tl in
                let frame = Int(tl.date.timeIntervalSinceReferenceDate / step)
                Canvas { ctx, size in
                    if scolding { drawTeacher(ctx, size, frame) }
                    else { drawDigger(ctx, size, frame) }
                }
                .frame(width: 224, height: 224)
                .background(Color(red: 0.86, green: 0.90, blue: 0.96))
            }
            .overlay(RoundedRectangle(cornerRadius: 2).stroke(.white, lineWidth: 4))

            Text(scolding ? "めっ！ もちだしきんし！" : "はっくつ ちゅう…")
                .font(.system(.headline, design: .monospaced)).bold()
                .foregroundStyle(scolding ? .yellow : .white)
            Text(scolding ? "おなじ いぶつは ２つと ない！" : "…ザクッ …ザクッ")
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.white.opacity(0.85)).multilineTextAlignment(.center)
        }
    }

    private var banner: some View {
        VStack(spacing: 12) {
            Text(scolding ? "！ もちだし ！" : "✨ しんはっけん！ ✨")
                .font(.system(size: 28, weight: .heavy, design: .monospaced))
                .foregroundStyle(scolding ? .red : .yellow)
                .shadow(color: .black, radius: 0, x: 2, y: 2)
                .multilineTextAlignment(.center)
            Text(scolding
                 ? "てんじしつ から かってに\nもちだしちゃった…！"
                 : "あたらしい いぶつ を\nてに いれた！")
                .font(.system(.headline, design: .monospaced))
                .foregroundStyle(.white).multilineTextAlignment(.center).lineSpacing(3)
        }
        .frame(height: 224)
        .frame(maxWidth: .infinity)
    }
}

// MARK: - ピクセル描画ヘルパ

private let cols: CGFloat = 32

private func filler(_ ctx: GraphicsContext, _ size: CGSize) -> (Int, Int, Int, Int, Color) -> Void {
    let cell = size.width / cols
    return { x, y, w, h, c in
        ctx.fill(Path(CGRect(x: CGFloat(x) * cell, y: CGFloat(y) * cell,
                             width: CGFloat(w) * cell, height: CGFloat(h) * cell)),
                 with: .color(c))
    }
}

// 色
private let outline = Color(red: 0.12, green: 0.10, blue: 0.18)
private let skin = Color(red: 1.0, green: 0.86, blue: 0.70)
private let hair = Color(red: 0.45, green: 0.27, blue: 0.13)
private let dress = Color(red: 0.88, green: 0.28, blue: 0.40)
private let metal = Color(red: 0.62, green: 0.64, blue: 0.70)
private let handle = Color(red: 0.55, green: 0.36, blue: 0.18)
private let dirt = Color(red: 0.50, green: 0.36, blue: 0.22)
private let dirtDark = Color(red: 0.36, green: 0.26, blue: 0.15)
private let pink = Color(red: 0.96, green: 0.58, blue: 0.60)
private let eye = Color(red: 0.13, green: 0.11, blue: 0.18)
private let anger = Color(red: 0.92, green: 0.14, blue: 0.14)
private let tHair = Color(red: 0.18, green: 0.14, blue: 0.11)
private let tSuit = Color(red: 0.30, green: 0.30, blue: 0.46)

// MARK: 女の子が採掘
private func drawDigger(_ ctx: GraphicsContext, _ size: CGSize, _ frame: Int) {
    let f = filler(ctx, size)
    let p = frame % 2
    let oy = p                       // 上下のバウンド

    // 地面
    f(0, 26, 32, 6, dirt)
    f(0, 26, 32, 1, Color(red: 0.58, green: 0.43, blue: 0.27))
    f(19, 25, 7, 3, dirtDark)        // 掘った穴

    // 髪
    f(10, 3 + oy, 9, 3, hair)
    f(9, 5 + oy, 2, 6, hair)
    f(18, 5 + oy, 2, 5, hair)
    // 顔
    f(11, 5 + oy, 7, 5, skin)
    f(12, 7 + oy, 1, 2, eye); f(16, 7 + oy, 1, 2, eye)      // 目
    f(11, 8 + oy, 1, 1, pink); f(17, 8 + oy, 1, 1, pink)   // ほっぺ
    f(14, 9 + oy, 1, 1, eye)                                // 口
    // 体（赤ワンピ）
    f(12, 10 + oy, 6, 2, dress)
    f(11, 12 + oy, 8, 3, dress)
    f(10, 15 + oy, 10, 2, dress)
    // 足
    f(12, 17 + oy, 2, 2, skin); f(16, 17 + oy, 2, 2, skin)
    // 後ろ腕
    f(10, 12 + oy, 2, 2, skin)

    // つるはし＆前腕
    if p == 0 {                      // 振り上げ
        f(18, 11 + oy, 2, 2, skin)
        f(20, 9 + oy, 2, 2, skin)
        f(22, 7 + oy, 2, 2, handle)
        f(23, 5 + oy, 2, 2, handle)
        f(22, 4 + oy, 5, 1, metal)
        f(25, 3 + oy, 2, 2, metal)
    } else {                         // 振り下ろし
        f(18, 13, 2, 2, skin)
        f(20, 15, 2, 2, skin)
        f(21, 17, 2, 2, handle)
        f(22, 19, 2, 2, handle)
        f(21, 21, 5, 1, metal)
        // 土ぼこり
        f(26, 19, 1, 1, .yellow); f(27, 17, 1, 1, .white)
        f(24, 16, 1, 1, .white);  f(20, 16, 1, 1, .yellow)
        f(25, 22, 2, 1, dirtDark)
    }
}

// MARK: 女の先生が叱る
private func drawTeacher(_ ctx: GraphicsContext, _ size: CGSize, _ frame: Int) {
    let f = filler(ctx, size)
    let p = frame % 2

    // 床
    f(0, 28, 32, 4, Color(red: 0.46, green: 0.42, blue: 0.52))

    // 髪＋お団子（横髪を長めにして女性らしく）
    f(11, 2, 10, 3, tHair)
    f(14, 0, 4, 2, tHair)
    f(10, 4, 2, 8, tHair); f(20, 4, 2, 8, tHair)
    f(9, 9, 1, 3, tHair); f(22, 9, 1, 3, tHair)            // 毛先
    // 顔
    f(12, 4, 8, 6, skin)
    // メガネ
    f(12, 6, 3, 2, .white); f(17, 6, 3, 2, .white)
    f(12, 6, 3, 1, eye); f(17, 6, 3, 1, eye)
    f(15, 6, 2, 1, eye)
    f(13, 7, 1, 1, eye); f(18, 7, 1, 1, eye)               // 鋭い目
    // 小さなへの字口
    f(15, 9, 2, 1, eye)
    // 体（スーツ）
    f(11, 10, 10, 3, tSuit)
    f(10, 13, 12, 5, tSuit)
    f(11, 18, 10, 3, tSuit)
    // 足
    f(12, 21, 3, 3, eye)
    if p == 0 { f(17, 21, 3, 3, eye) } else { f(17, 20, 3, 3, eye) }   // 足踏み

    // 指さし腕（プルプル）
    let ax = p == 0 ? 0 : 1
    f(21, 11, 2, 2, tSuit)
    f(23 + ax, 10, 2, 2, skin)
    f(25 + ax, 9, 3, 1, skin)

    // 怒りマーク💢（点滅）
    if p == 0 {
        f(22, 1, 1, 1, anger); f(24, 1, 1, 1, anger)
        f(23, 2, 1, 1, anger)
        f(22, 3, 1, 1, anger); f(24, 3, 1, 1, anger)
        f(21, 0, 1, 1, anger); f(25, 4, 1, 1, anger)
    } else {
        f(6, 3, 1, 1, anger); f(8, 3, 1, 1, anger); f(7, 4, 1, 1, anger)
        f(6, 5, 1, 1, anger); f(8, 5, 1, 1, anger)
    }
}
