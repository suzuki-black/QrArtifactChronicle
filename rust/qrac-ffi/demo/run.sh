#!/usr/bin/env bash
# Rust(qrac-core) を UniFFI 経由で Swift から呼ぶ最小デモのビルド&実行。
# 使い方: rust/ ディレクトリで  bash qrac-ffi/demo/run.sh
set -euo pipefail
cd "$(dirname "$0")/../.."   # → rust/

echo "1) Rust cdylib をビルド"
cargo build -p qrac-ffi

echo "2) Swift バインディングを生成（uniffi-bindgen, 同一uniffi版）"
cargo run -q -p qrac-ffi --bin uniffi-bindgen -- \
  generate --library target/debug/libqrac_ffi.dylib --language swift --out-dir target/swift

echo "3) Swift デモをコンパイル"
swiftc -O \
  -I target/swift -Xcc -fmodule-map-file=target/swift/qrac_ffiFFI.modulemap -Xcc -Itarget/swift \
  -L target/debug -lqrac_ffi \
  target/swift/qrac_ffi.swift qrac-ffi/demo/main.swift \
  -o target/qrac-demo

echo "4) 実行"
DYLD_LIBRARY_PATH=target/debug ./target/qrac-demo
