use std::{fs, path::Path};

pub fn create_folders(root: &Path, paths: Vec<&str>) -> std::io::Result<()> {
    for path in paths {
        fs::create_dir_all(root.join(path))?
    }
    Ok(())
}
