// Compile proto/*.proto into Rust via prost. We use the vendored protoc
// binary so cross-platform builds don't need a system install.
fn main() -> std::io::Result<()> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    println!("cargo:rerun-if-changed=proto/events.proto");
    prost_build::compile_protos(&["proto/events.proto"], &["proto"])
}
