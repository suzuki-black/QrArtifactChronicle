// 同一 uniffi 版でバインディングを生成するエントリ。
// 例: cargo run -p qrac-ffi --bin uniffi-bindgen -- generate \
//        --library target/debug/libqrac_ffi.dylib --language swift --out-dir target/swift
fn main() {
    uniffi::uniffi_bindgen_main()
}
