import SwiftUI
import AVFoundation
import AppKit

/// QRスキャンページ（A: ページ遷移）。ライブプレビュー＋ファインダー枠。
/// 検出すると onScan（撮影不要・自動）。✕ で onCancel。
struct CameraScanView: View {
    let onScan: (String) -> Void
    let onCancel: () -> Void
    @EnvironmentObject var settings: Settings
    @StateObject private var scanner = Scanner()

    var body: some View {
        VStack(spacing: 0) {
            header
            ZStack {
                Color.black
                content
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color.black)
        .onAppear {
            scanner.onScan = { s in onScan(s) }
            scanner.start()
        }
        .onDisappear { scanner.stop() }
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button { scanner.stop(); onCancel() } label: {
                Label(settings.t("とじる", "Close"), systemImage: "xmark").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.gray).controlSize(.small)
            Text(settings.t("QRを読み取る", "Scan QR")).font(.title3.bold())
            Spacer()
            cameraMenu
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private var currentCameraName: String {
        scanner.devices.first { $0.uniqueID == scanner.currentID }?.localizedName
            ?? settings.t("カメラ", "Camera")
    }

    private var cameraMenu: some View {
        Menu {
            ForEach(scanner.devices, id: \.uniqueID) { d in
                Button {
                    scanner.select(d)
                } label: {
                    Label(d.localizedName,
                          systemImage: d.uniqueID == scanner.currentID ? "checkmark" : "video")
                }
            }
            Divider()
            Button { scanner.refreshDevices() } label: {
                Label(settings.t("カメラを再検索", "Re-scan cameras"), systemImage: "arrow.clockwise")
            }
        } label: {
            Label(currentCameraName, systemImage: "video.fill")
                .font(.caption).lineLimit(1)
        }
        .menuStyle(.borderlessButton)
        .fixedSize()
    }

    @ViewBuilder private var content: some View {
        switch scanner.state {
        case .running:
            CameraPreview(session: scanner.session)
            reticle
        case .authorizing, .idle:
            ProgressView(settings.t("カメラを準備中…", "Preparing camera…"))
                .tint(.white).foregroundStyle(.white)
        case .denied:
            message(settings.t("カメラの使用が許可されていません", "Camera access is not allowed"),
                    settings.t("システム設定 ＞ プライバシーとセキュリティ ＞ カメラ で本アプリを許可してください。",
                               "Allow this app under System Settings ＞ Privacy & Security ＞ Camera."))
        case .noCamera:
            message(settings.t("カメラが見つかりません", "No camera found"),
                    settings.t("この端末で使えるカメラがありません。", "No usable camera on this device."))
        case .error(let m):
            message(settings.t("エラー", "Error"), m)
        }
    }

    private var reticle: some View {
        VStack(spacing: 16) {
            Spacer()
            RoundedRectangle(cornerRadius: 16)
                .stroke(.white, lineWidth: 3)
                .frame(width: 220, height: 220)
                .background(Color.white.opacity(0.04))
            Text(settings.t("QRコードを枠内に収めてください", "Fit the QR code in the frame"))
                .font(.callout.bold()).foregroundStyle(.white)
                .padding(.horizontal, 12).padding(.vertical, 6)
                .background(.black.opacity(0.5)).clipShape(Capsule())
            #if DEBUG
            Text("解析フレーム: \(scanner.frames)")
                .font(.caption2.monospaced()).foregroundStyle(.white.opacity(0.7))
            #endif
            Spacer()
        }
        .padding()
    }

    private func message(_ title: String, _ body: String) -> some View {
        VStack(spacing: 10) {
            Image(systemName: "camera.fill").font(.system(size: 44)).foregroundStyle(.white.opacity(0.7))
            Text(title).font(.headline).foregroundStyle(.white)
            Text(body).font(.callout).foregroundStyle(.white.opacity(0.8))
                .multilineTextAlignment(.center)
        }
        .padding(24)
    }
}

/// AVCaptureVideoPreviewLayer を表示する NSView ラッパ。
struct CameraPreview: NSViewRepresentable {
    let session: AVCaptureSession
    func makeNSView(context: Context) -> PreviewView {
        let v = PreviewView()
        v.attach(session)
        return v
    }
    func updateNSView(_ nsView: PreviewView, context: Context) {}
}

final class PreviewView: NSView {
    private let previewLayer = AVCaptureVideoPreviewLayer()
    override init(frame: NSRect) { super.init(frame: frame); setup() }
    required init?(coder: NSCoder) { super.init(coder: coder); setup() }
    private func setup() {
        previewLayer.videoGravity = .resizeAspectFill
        layer = previewLayer
        wantsLayer = true
    }
    func attach(_ session: AVCaptureSession) { previewLayer.session = session }
    override func layout() {
        super.layout()
        previewLayer.frame = bounds
    }
}
