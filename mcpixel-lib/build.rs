fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=../proto/block.proto");
    prost_build::compile_protos(&["../proto/block.proto"], &["../proto/"])?;
    Ok(())
}
