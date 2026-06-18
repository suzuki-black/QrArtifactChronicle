import SwiftUI

#if os(macOS)
import AppKit
/// プラットフォーム画像型（macOS=NSImage / iOS=UIImage）。
typealias PlatformImage = NSImage
#else
import UIKit
typealias PlatformImage = UIImage
#endif

extension Image {
    /// プラットフォーム画像から SwiftUI Image を作る。
    init(platformImage img: PlatformImage) {
        #if os(macOS)
        self.init(nsImage: img)
        #else
        self.init(uiImage: img)
        #endif
    }
}

/// プラットフォーム差のある画像処理をまとめる。
enum PlatformGfx {
    /// PNG データを最大 maxDim(px) へ縮小して PNG を返す。失敗時は原本をそのまま返す。
    static func downscalePNG(_ data: Data, maxDim: CGFloat) -> Data {
        #if os(macOS)
        guard let src = NSImage(data: data) else { return data }
        let s = src.size
        guard s.width > 0, s.height > 0 else { return data }
        let scale = min(1, maxDim / max(s.width, s.height))
        let w = Int((s.width * scale).rounded()), h = Int((s.height * scale).rounded())
        guard let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: w, pixelsHigh: h,
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0) else { return data }
        rep.size = NSSize(width: w, height: h)
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
        src.draw(in: NSRect(x: 0, y: 0, width: w, height: h))
        NSGraphicsContext.restoreGraphicsState()
        return rep.representation(using: .png, properties: [:]) ?? data
        #else
        guard let src = UIImage(data: data) else { return data }
        let s = src.size
        guard s.width > 0, s.height > 0 else { return data }
        let scale = min(1, maxDim / max(s.width, s.height))
        let target = CGSize(width: s.width * scale, height: s.height * scale)
        let fmt = UIGraphicsImageRendererFormat.default()
        fmt.scale = 1 // 1pt=1px（決め打ちサイズで保存）
        return UIGraphicsImageRenderer(size: target, format: fmt).pngData { _ in
            src.draw(in: CGRect(origin: .zero, size: target))
        }
        #endif
    }
}
