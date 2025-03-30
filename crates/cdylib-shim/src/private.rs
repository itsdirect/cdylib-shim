use std::path::PathBuf;

pub use libloading::{Error, Library, Symbol};

#[cfg(windows)]
pub fn system_dir() -> Option<PathBuf> {
    let system_root = std::env::var("SYSTEMROOT").ok()?;
    let path = PathBuf::from(format!("{}\\System32", system_root));
    Some(path)
}

#[cfg(not(windows))]
pub fn system_dir() -> Option<PathBuf> {
    None
}
