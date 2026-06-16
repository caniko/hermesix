use crate::redaction::{kind_name, parse_by_kind, portable_policy_check, FileKind, Redaction};
use clap::Args;
use miette::{bail, miette, Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct Diff {
    #[arg(long)]
    pub manifest: PathBuf,

    #[arg(long)]
    pub config_dir: PathBuf,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct Sync {
    #[arg(long)]
    pub manifest: PathBuf,

    #[arg(long)]
    pub config_dir: PathBuf,

    #[arg(long)]
    pub apply: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct Validate {
    #[arg(long)]
    pub manifest: PathBuf,

    #[arg(long)]
    pub config_dir: PathBuf,

    #[command(flatten)]
    pub redaction: Redaction,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    version: u64,
    module: Option<String>,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    path: String,
    source: PathBuf,
    target: PathBuf,
    sha256: String,
    kind: FileKind,
    origin: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum FileStatus {
    Same,
    Missing,
    Changed,
}

#[derive(Debug, Serialize)]
struct DiffEntry {
    status: FileStatus,
    path: String,
}

pub fn diff_command(args: Diff) -> Result<i32> {
    let manifest = read_manifest(&args.manifest)?;
    require_valid_manifest_shape(&manifest, &args.config_dir)?;
    let entries = diff_entries(&manifest, &args.config_dir)?;
    print_diff(&entries, args.json)?;
    Ok(if entries.is_empty() { 0 } else { 1 })
}

pub fn sync_command(args: Sync) -> Result<i32> {
    let manifest = read_manifest(&args.manifest)?;
    require_valid_manifest_shape(&manifest, &args.config_dir)?;
    let entries = diff_entries(&manifest, &args.config_dir)?;
    print_diff(&entries, args.json)?;
    if entries.is_empty() {
        return Ok(0);
    }
    if args.apply {
        for entry in &entries {
            let file = manifest
                .files
                .iter()
                .find(|file| file.path == entry.path)
                .ok_or_else(|| miette!("manifest lost entry for {}", entry.path))?;
            verify_source_hash(file)?;
            copy_atomic(&file.source, &args.config_dir.join(&file.path))?;
        }
        Ok(0)
    } else {
        if !args.json {
            println!("dry-run: pass --apply to write changes");
        }
        Ok(1)
    }
}

pub fn validate_command(args: Validate) -> Result<()> {
    let manifest = read_manifest(&args.manifest)?;
    let mut errors = manifest_shape_errors(&manifest, &args.config_dir);
    for file in &manifest.files {
        if !file.source.exists() {
            errors.push(format!("{}: source does not exist", file.path));
            continue;
        }
        match verify_source_hash(file) {
            Ok(()) => {}
            Err(err) => errors.push(format!("{}: {err:?}", file.path)),
        }
        if let Err(err) = parse_by_kind(&file.source, &file.path, file.kind) {
            errors.push(format!(
                "{}: cannot parse {}: {err:?}",
                file.path,
                kind_name(file.kind)
            ));
        }
        if let Err(err) =
            portable_policy_check(&file.source, &file.path, file.kind, &args.redaction)
        {
            errors.push(format!("{}: {err:?}", file.path));
        }
    }
    if errors.is_empty() {
        println!("manifest ok");
        Ok(())
    } else {
        for err in errors {
            eprintln!("{err}");
        }
        bail!("validation failed");
    }
}

fn require_valid_manifest_shape(manifest: &Manifest, config_dir: &Path) -> Result<()> {
    let errors = manifest_shape_errors(manifest, config_dir);
    if errors.is_empty() {
        Ok(())
    } else {
        for err in errors {
            eprintln!("{err}");
        }
        bail!("invalid manifest")
    }
}

fn manifest_shape_errors(manifest: &Manifest, config_dir: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    if manifest.version != 1 {
        errors.push(format!("unsupported manifest version {}", manifest.version));
    }
    if matches!(manifest.module.as_deref(), Some("")) {
        errors.push("manifest module must not be empty".to_string());
    }
    for file in &manifest.files {
        if Path::new(&file.path).is_absolute() || file.path.split('/').any(|part| part == "..") {
            errors.push(format!(
                "{} ({}): relative path escapes config root",
                file.path, file.origin
            ));
        }
        let expected_target = config_dir.join(&file.path);
        if file.target != expected_target {
            errors.push(format!(
                "{} ({}): manifest target {} does not match config dir target {}",
                file.path,
                file.origin,
                file.target.display(),
                expected_target.display()
            ));
        }
    }
    errors
}

fn verify_source_hash(file: &ManifestFile) -> Result<()> {
    match sha256_file(&file.source) {
        Ok(hash) if hash == file.sha256 => Ok(()),
        Ok(hash) => bail!("source sha256 mismatch: {hash}"),
        Err(err) => {
            Err(err).wrap_err_with(|| format!("cannot hash source {}", file.source.display()))
        }
    }
}

fn diff_entries(manifest: &Manifest, config_dir: &Path) -> Result<Vec<DiffEntry>> {
    let mut entries = Vec::new();
    for file in &manifest.files {
        let status = file_status(file, config_dir)?;
        if status != FileStatus::Same {
            entries.push(DiffEntry {
                status,
                path: file.path.clone(),
            });
        }
    }
    Ok(entries)
}

fn print_diff(entries: &[DiffEntry], json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(entries).into_diagnostic()?
        );
    } else if entries.is_empty() {
        println!("no changes");
    } else {
        for entry in entries {
            println!("{} {}", status_name(&entry.status), entry.path);
        }
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<Manifest> {
    serde_json::from_reader(
        fs::File::open(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("cannot open manifest {}", path.display()))?,
    )
    .into_diagnostic()
}

fn file_status(file: &ManifestFile, config_dir: &Path) -> Result<FileStatus> {
    let target = config_dir.join(&file.path);
    if !target.exists() {
        return Ok(FileStatus::Missing);
    }
    let target_hash = sha256_file(&target)?;
    if target_hash == file.sha256 {
        Ok(FileStatus::Same)
    } else {
        Ok(FileStatus::Changed)
    }
}

fn status_name(status: &FileStatus) -> &'static str {
    match status {
        FileStatus::Same => "same",
        FileStatus::Missing => "missing",
        FileStatus::Changed => "changed",
    }
}

fn copy_atomic(source: &Path, target: &Path) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| miette!("target `{}` has no parent", target.display()))?;
    fs::create_dir_all(parent).into_diagnostic()?;
    let tmp = target.with_extension(format!(
        "{}.tmp.{}",
        target.extension().and_then(OsStr::to_str).unwrap_or("file"),
        std::process::id()
    ));
    fs::copy(source, &tmp).into_diagnostic()?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode_unix(0o644)).into_diagnostic()?;
    fs::rename(tmp, target).into_diagnostic()?;
    Ok(())
}

#[cfg(unix)]
trait PermissionsExtUnix {
    fn from_mode_unix(mode: u32) -> Self;
}

#[cfg(unix)]
impl PermissionsExtUnix for fs::Permissions {
    fn from_mode_unix(mode: u32) -> Self {
        use std::os::unix::fs::PermissionsExt;
        fs::Permissions::from_mode(mode)
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).into_diagnostic()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::env;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEST_DIR: AtomicUsize = AtomicUsize::new(0);

    fn temp_root(name: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let root =
            env::temp_dir().join(format!("hermesix-test-{}-{id}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_file(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn write_manifest(
        root: &Path,
        config_dir: &Path,
        rel_path: &str,
        source: &Path,
        sha256: &str,
    ) -> PathBuf {
        let manifest = root.join("manifest.json");
        let value = json!({
            "version": 1,
            "module": "programs.test",
            "files": [{
                "path": rel_path,
                "source": source,
                "target": config_dir.join(rel_path),
                "sha256": sha256,
                "kind": "raw",
                "origin": "test"
            }]
        });
        fs::write(&manifest, serde_json::to_string_pretty(&value).unwrap()).unwrap();
        manifest
    }

    #[test]
    fn sync_apply_writes_valid_manifest_and_diff_cleans() {
        let root = temp_root("valid-sync");
        let source = root.join("source/settings.txt");
        let live = root.join("live");
        write_file(&source, "managed\n");
        let hash = sha256_file(&source).unwrap();
        let manifest = write_manifest(&root, &live, "settings.txt", &source, &hash);

        let diff_status = diff_command(Diff {
            manifest: manifest.clone(),
            config_dir: live.clone(),
            json: true,
        })
        .unwrap();
        assert_eq!(diff_status, 1);

        let sync_status = sync_command(Sync {
            manifest: manifest.clone(),
            config_dir: live.clone(),
            apply: true,
            json: true,
        })
        .unwrap();
        assert_eq!(sync_status, 0);
        assert_eq!(
            fs::read_to_string(live.join("settings.txt")).unwrap(),
            "managed\n"
        );

        let clean_status = diff_command(Diff {
            manifest,
            config_dir: live,
            json: true,
        })
        .unwrap();
        assert_eq!(clean_status, 0);
    }

    #[test]
    fn diff_rejects_manifest_path_traversal() {
        let root = temp_root("diff-traversal");
        let source = root.join("source/settings.txt");
        let live = root.join("live");
        write_file(&source, "managed\n");
        let hash = sha256_file(&source).unwrap();
        let manifest = write_manifest(&root, &live, "../escape.txt", &source, &hash);

        let result = diff_command(Diff {
            manifest,
            config_dir: live,
            json: true,
        });
        assert!(result.is_err());
    }

    #[test]
    fn sync_apply_rejects_manifest_path_traversal() {
        let root = temp_root("sync-traversal");
        let source = root.join("source/settings.txt");
        let live = root.join("live");
        write_file(&source, "managed\n");
        let hash = sha256_file(&source).unwrap();
        let manifest = write_manifest(&root, &live, "../escape.txt", &source, &hash);

        let result = sync_command(Sync {
            manifest,
            config_dir: live,
            apply: true,
            json: true,
        });
        assert!(result.is_err());
    }

    #[test]
    fn sync_apply_rejects_source_hash_mismatch() {
        let root = temp_root("hash-mismatch");
        let source = root.join("source/settings.txt");
        let live = root.join("live");
        write_file(&source, "managed\n");
        let manifest = write_manifest(&root, &live, "settings.txt", &source, "not-the-hash");

        let result = sync_command(Sync {
            manifest,
            config_dir: live,
            apply: true,
            json: true,
        });
        assert!(result.is_err());
    }
}
