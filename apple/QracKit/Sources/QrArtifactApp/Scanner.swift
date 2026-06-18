import AVFoundation
import Vision
import Foundation

/// QRライブスキャナ。映像フレームを Vision(VNDetectBarcodesRequest) で解析して QR を検出。
/// AVCaptureMetadataOutput を使わないため、iPhone連係カメラなど機種を問わず動作する。
final class Scanner: NSObject, ObservableObject {
    enum ScanState: Equatable {
        case idle, authorizing, running, denied, noCamera
        case error(String)
    }

    @Published var state: ScanState = .idle
    @Published var devices: [AVCaptureDevice] = []
    @Published var currentID: String?
    @Published var frames: Int = 0          // 解析フレーム数（デバッグ表示用）

    let session = AVCaptureSession()
    var onScan: ((String) -> Void)?

    private let sessionQueue = DispatchQueue(label: "qrac.scanner.session")
    private let videoQueue = DispatchQueue(label: "qrac.scanner.video")
    private let videoOutput = AVCaptureVideoDataOutput()
    private var didScan = false
    private var frameSkip = 0
    private var selected: AVCaptureDevice?

    override init() {
        super.init()
        // カメラの接続/切断（iPhone連係の出入りなど）を検知して一覧を自動更新。
        let nc = NotificationCenter.default
        nc.addObserver(self, selector: #selector(devicesChanged),
                       name: AVCaptureDevice.wasConnectedNotification, object: nil)
        nc.addObserver(self, selector: #selector(devicesChanged),
                       name: AVCaptureDevice.wasDisconnectedNotification, object: nil)
    }
    deinit { NotificationCenter.default.removeObserver(self) }

    @objc private func devicesChanged() { refreshDevices() }

    // MARK: 開始・権限

    func start() {
        didScan = false
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            begin()
        case .notDetermined:
            setState(.authorizing)
            AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
                granted ? self?.begin() : self?.setState(.denied)
            }
        default:
            setState(.denied)
        }
    }

    func stop() {
        sessionQueue.async { [weak self] in
            guard let self, self.session.isRunning else { return }
            self.session.stopRunning()
        }
    }

    private func begin() {
        refreshDevices()
        let d = selected ?? discover().first ?? AVCaptureDevice.default(for: .video)
        selected = d
        configure(d)
    }

    // MARK: デバイス一覧・切替

    private func discover() -> [AVCaptureDevice] {
        #if os(macOS)
        // macOS: 内蔵＋連係カメラ(iPhone)＋外付け。位置は問わない。
        var types: [AVCaptureDevice.DeviceType] = [.builtInWideAngleCamera]
        if #available(macOS 14.0, *) {
            types = [.continuityCamera, .external, .builtInWideAngleCamera]
        }
        return AVCaptureDevice.DiscoverySession(
            deviceTypes: types, mediaType: .video, position: .unspecified).devices
        #else
        // iOS: 背面の広角カメラを既定に（QR向き）。
        return AVCaptureDevice.DiscoverySession(
            deviceTypes: [.builtInWideAngleCamera], mediaType: .video, position: .back).devices
        #endif
    }

    func refreshDevices() {
        let list = discover()
        DispatchQueue.main.async { self.devices = list }
    }

    func select(_ device: AVCaptureDevice) {
        selected = device
        configure(device)
    }

    // MARK: セッション構成

    private func setState(_ s: ScanState) { DispatchQueue.main.async { self.state = s } }

    private func configureFocus(_ device: AVCaptureDevice) {
        guard (try? device.lockForConfiguration()) != nil else { return }
        if device.isFocusModeSupported(.continuousAutoFocus) {
            device.focusMode = .continuousAutoFocus
        }
        device.unlockForConfiguration()
    }

    private func configure(_ device: AVCaptureDevice?) {
        guard let device else { setState(.noCamera); return }
        sessionQueue.async { [weak self] in
            guard let self else { return }
            if self.session.isRunning { self.session.stopRunning() }
            self.session.beginConfiguration()
            self.session.inputs.forEach { self.session.removeInput($0) }
            self.session.outputs.forEach { self.session.removeOutput($0) }

            guard let input = try? AVCaptureDeviceInput(device: device),
                  self.session.canAddInput(input) else {
                self.session.commitConfiguration()
                self.setState(.error("カメラ入力を開けませんでした")); return
            }
            self.session.addInput(input)
            if self.session.canSetSessionPreset(.high) { self.session.sessionPreset = .high }
            self.configureFocus(device)

            self.videoOutput.alwaysDiscardsLateVideoFrames = true
            self.videoOutput.setSampleBufferDelegate(self, queue: self.videoQueue)
            guard self.session.canAddOutput(self.videoOutput) else {
                self.session.commitConfiguration()
                self.setState(.error("映像出力を追加できませんでした")); return
            }
            self.session.addOutput(self.videoOutput)

            self.session.commitConfiguration()
            self.didScan = false
            self.session.startRunning()
            DispatchQueue.main.async {
                self.currentID = device.uniqueID
                self.state = .running
            }
        }
    }
}

extension Scanner: AVCaptureVideoDataOutputSampleBufferDelegate {
    func captureOutput(_ output: AVCaptureOutput,
                       didOutput sampleBuffer: CMSampleBuffer,
                       from connection: AVCaptureConnection) {
        guard !didScan else { return }

        // 負荷軽減: 数フレームに1回だけ解析
        frameSkip += 1
        if frameSkip % 3 != 0 { return }
        let n = frameSkip
        DispatchQueue.main.async { self.frames = n / 3 }

        guard let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        let request = VNDetectBarcodesRequest()
        request.symbologies = [.qr]
        let handler = VNImageRequestHandler(cvPixelBuffer: pixelBuffer, options: [:])
        try? handler.perform([request])

        guard let result = request.results?.first as? VNBarcodeObservation,
              let payload = result.payloadStringValue, !payload.isEmpty else { return }

        didScan = true
        stop()
        DispatchQueue.main.async { [weak self] in self?.onScan?(payload) }
    }
}
