use std::path::Path;

use crate::util::file::create_folders;

pub fn init_data(root_folder: &Path) -> std::io::Result<()> {
    create_folders(
        root_folder,
        vec![
            "frontend",
            "covers",
            "mangas",
            "ssl",
            "temp",
            "external",
            "users/banner",
            "users/icon",
            "cover_templates",
        ],
    )
}
