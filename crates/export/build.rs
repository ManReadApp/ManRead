fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/manga.proto");
    println!("cargo:rerun-if-changed=proto");

    prost_build::compile_protos(&["proto/manga.proto"], &["proto"])?;
    Ok(())
}
