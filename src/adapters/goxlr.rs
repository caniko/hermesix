//! GoXLR Utility capture adapter.
//!
//! The adapter deliberately copies the native GoXLR Utility artifacts without
//! interpreting their contents.  The resulting manifest is consumed by the
//! `goxlr-config` reconciler from the goxlr-nexus flake, so profile evolution
//! does not require a Hermesix release to preserve new fields.

use crate::redaction::FileKind;
use clap::{Args, Subcommand};
use miette::{bail, Context, IntoDiagnostic, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_VERSION: u64 = 1;
const MANIFEST_MODULE: &str = "programs.goxlr-utility";

#[derive(Debug, Subcommand)]
pub enum GoxlrCommand {
    /// Capture settings and native profile assets from the live XDG tree.
    Capture(Capture),
}

#[derive(Debug, Args)]
pub struct Capture {
    /// Destination directory for the declarative source tree and manifest.
    #[arg(long)]
    pub output_dir: PathBuf,

    /// Live GoXLR Utility configuration directory.
    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    /// Live GoXLR Utility data directory.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Runtime target for settings.json; defaults to --config-dir.
    #[arg(long)]
    pub target_config_dir: Option<PathBuf>,

    /// Runtime target for profiles/assets; defaults to --data-dir.
    #[arg(long)]
    pub target_data_dir: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct Manifest {
    version: u64,
    module: &'static str,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Serialize)]
struct ManifestFile {
    path: String,
    source: PathBuf,
    target: PathBuf,
    sha256: String,
    kind: FileKind,
    origin: String,
}

pub fn run(command: GoxlrCommand) -> Result<i32> {
    match command {
        GoxlrCommand::Capture(args) => {
            capture(args)?;
            Ok(0)
        }
    }
}

fn capture(args: Capture) -> Result<()> {
    let config_dir = args.config_dir.unwrap_or_else(default_config_dir);
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let target_config_dir = args
        .target_config_dir
        .clone()
        .unwrap_or_else(|| config_dir.clone());
    let target_data_dir = args
        .target_data_dir
        .clone()
        .unwrap_or_else(|| data_dir.clone());
    let assets_root = args.output_dir.join("assets");
    fs::create_dir_all(&assets_root)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot create {}", assets_root.display()))?;

    let mut files = Vec::new();
    let settings = config_dir.join("settings.json");
    if settings.is_file() {
        files.push(copy_artifact(
            &settings,
            &assets_root.join("settings.json"),
            "settings.json",
            target_config_dir.join("settings.json"),
            FileKind::Json,
            "goxlr-utility:settings.json",
        )?);
    }

    for (directory, kind) in [
        ("profiles", FileKind::Raw),
        ("mic-profiles", FileKind::Raw),
        ("presets", FileKind::Raw),
        ("samples", FileKind::Raw),
        ("icons", FileKind::Raw),
    ] {
        let source_root = data_dir.join(directory);
        if !source_root.is_dir() {
            continue;
        }
        for source in sorted_files(&source_root)? {
            let relative = source
                .strip_prefix(&source_root)
                .into_diagnostic()?
                .to_string_lossy()
                .replace('\\', "/");
            if relative.is_empty() || relative.split('/').any(|part| part == "..") {
                bail!("invalid GoXLR artifact path {relative}");
            }
            let logical = format!("{directory}/{relative}");
            files.push(copy_artifact(
                &source,
                &assets_root.join(&logical),
                &logical,
                target_data_dir.join(&logical),
                kind,
                &format!("goxlr-utility:{logical}"),
            )?);
        }
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = Manifest {
        version: MANIFEST_VERSION,
        module: MANIFEST_MODULE,
        files,
    };
    let manifest_path = args.output_dir.join("goxlr-config-manifest.json");
    let manifest_json = serde_json::to_vec_pretty(&manifest)
        .into_diagnostic()
        .context("cannot serialize GoXLR manifest")?;
    fs::write(&manifest_path, format!("{}\n", String::from_utf8_lossy(&manifest_json)))
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot write {}", manifest_path.display()))?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&manifest)
                .into_diagnostic()
                .context("cannot serialize capture result")?
        );
    } else {
        println!(
            "captured {} GoXLR artifact(s) into {}",
            manifest.files.len(),
            args.output_dir.display()
        );
        println!("manifest: {}", manifest_path.display());
    }
    Ok(())
}

fn copy_artifact(
    source: &Path,
    destination: &Path,
    logical: &str,
    target: PathBuf,
    kind: FileKind,
    origin: &str,
) -> Result<ManifestFile> {
    if !source.is_file() {
        bail!("GoXLR artifact is not a regular file: {}", source.display());
    }
    if !target.is_absolute() {
        bail!("GoXLR runtime target must be absolute: {}", target.display());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .into_diagnostic()
            .wrap_err_with(|| format!("cannot create {}", parent.display()))?;
    }
    fs::copy(source, destination)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot capture {}", source.display()))?;
    Ok(ManifestFile {
        path: logical.to_string(),
        source: destination.to_path_buf(),
        target,
        sha256: sha256(destination)?,
        kind,
        origin: origin.to_string(),
    })
}

fn sorted_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot read {}", root.display()))?
    {
        let path = entry.into_diagnostic()?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot read {}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn default_xdg_dir(variable: &str, suffix: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(suffix)))
        .unwrap_or_else(|| PathBuf::from(suffix))
        .join("goxlr-utility")
}

fn default_config_dir() -> PathBuf {
    default_xdg_dir("XDG_CONFIG_HOME", ".config")
}

fn default_data_dir() -> PathBuf {
    default_xdg_dir("XDG_DATA_HOME", ".local/share")
}
