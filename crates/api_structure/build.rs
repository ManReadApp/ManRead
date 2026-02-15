use std::{
    fs,
    path::{Path, PathBuf},
};

fn collect_proto_files(root: impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let ty = entry.file_type()?;

            if ty.is_dir() {
                walk(&path, out)?;
            } else if ty.is_file() && path.extension().and_then(|e| e.to_str()) == Some("proto") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    walk(root.as_ref(), &mut files)?;

    files.sort();
    Ok(files)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = prost_build::Config::new();
    println!("cargo:rerun-if-changed=proto/v1");
    cfg.message_attribute(
            ".",
            "#[derive(serde::Serialize, serde::Deserialize, apistos::ApiComponent, schemars::JsonSchema)]",
        );
    cfg.enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize, apistos::ApiComponent, schemars::JsonSchema)]");
    cfg.set_proto_enums_as_rust_enums(true);
    let protos = collect_proto_files("proto/v1")?;
    for p in &protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }
    cfg.compile_protos(&protos, &["proto"])?;
    Ok(())
}
