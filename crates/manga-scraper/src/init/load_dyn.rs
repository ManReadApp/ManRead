use std::{fs, sync::Arc};

use libloading::Library;
use scraper_module::{req::ClientT, Functions, RegisterOverride};

use crate::runtime::{fetch::ReqwestClient, files::guess_episode};

type LoadFn = unsafe fn(client: Arc<dyn ClientT>, functions: Functions) -> RegisterOverride;

pub fn load_dyn(dir: &std::path::Path) -> Vec<RegisterOverride> {
    let mut overrides = Vec::new();
    let functions = Functions {
        guess_episode: guess_episode,
    };
    for path in fs::read_dir(dir.join("external"))
        .map(|v| {
            v.filter_map(|v| v.ok().map(|v| v.path()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
    {
        if path
            .extension()
            .map_or(false, |ext| ext == "dll" || ext == "dylib" || ext == "so")
        {
            unsafe {
                if let Ok(lib) = Library::new(&path) {
                    if let Ok(load_fn) = lib.get::<LoadFn>(b"load") {
                        overrides.push(load_fn(Arc::new(ReqwestClient::new()), functions.clone()));
                    }
                }
            }
        }
    }
    //TODO: overrides
    overrides
}
