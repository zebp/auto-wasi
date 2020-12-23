use auto_wasi::WasiVersion;

#[test]
fn detect_snapshot_0() {
    let binary = include_bytes!("data/snapshot_0.wasm");
    let version = WasiVersion::detect(binary).expect("invalid wasm binary");
    assert_eq!(version, WasiVersion::Snapshot0);
}

#[test]
fn detect_snapshot_1() {
    let binary = include_bytes!("data/snapshot_1.wasm");
    let version = WasiVersion::detect(binary).expect("invalid wasm binary");
    assert_eq!(version, WasiVersion::Snapshot1);
}