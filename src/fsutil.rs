use miette::{miette, IntoDiagnostic, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub fn sorted_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)
        .into_diagnostic()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

pub fn sorted_files_with_ext(root: &Path, ext: &str) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)
        .into_diagnostic()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some(ext))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

pub fn sorted_recursive_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_files(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

pub fn path_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(str::to_string)
        .ok_or_else(|| miette!("invalid path `{}`", path.display()))
}

fn collect_files(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if root.is_dir() {
        for entry in fs::read_dir(root).into_diagnostic()? {
            let path = entry.into_diagnostic()?.path();
            if path.is_dir() {
                collect_files(&path, paths)?;
            } else if path.is_file() {
                paths.push(path);
            }
        }
    }
    Ok(())
}
