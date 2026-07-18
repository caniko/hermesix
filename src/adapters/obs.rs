use crate::fsutil::{path_name, sorted_dirs, sorted_files_with_ext, sorted_recursive_files};
use crate::nix::to_nix;
use crate::redaction::{ini_to_json, sanitized_json_file, Redaction};
use clap::{Args, Subcommand};
use miette::{bail, miette, Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Subcommand)]
pub enum ObsCommand {
    ExportToNix(ExportToNix),
    PluginInspect(PluginInspect),
}

#[derive(Debug, Args)]
pub struct ExportToNix {
    pub config_dir: Option<PathBuf>,

    #[command(flatten)]
    pub redaction: Redaction,
}

#[derive(Debug, Args)]
pub struct PluginInspect {
    #[command(subcommand)]
    pub command: Option<PluginInspectCommand>,

    #[arg(long, conflicts_with = "install_dir")]
    pub source_dir: Option<PathBuf>,

    #[arg(long, conflicts_with = "source_dir")]
    pub install_dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum PluginInspectCommand {
    Verify(PluginInspectVerify),
}

#[derive(Debug, Args)]
pub struct PluginInspectVerify {
    #[arg(long)]
    pub evidence: PathBuf,

    #[arg(long, conflicts_with = "install_dir")]
    pub source_dir: Option<PathBuf>,

    #[arg(long, conflicts_with = "source_dir")]
    pub install_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct PluginEvidence {
    source_ids: Vec<EvidenceLine>,
    filter_ids: Vec<EvidenceLine>,
    output_ids: Vec<EvidenceLine>,
    encoder_ids: Vec<EvidenceLine>,
    registrations: Vec<EvidenceLine>,
    setting_defaults: Vec<EvidenceLine>,
    property_settings: Vec<EvidenceLine>,
}

impl PluginEvidence {
    fn empty() -> Self {
        Self {
            source_ids: Vec::new(),
            filter_ids: Vec::new(),
            output_ids: Vec::new(),
            encoder_ids: Vec::new(),
            registrations: Vec::new(),
            setting_defaults: Vec::new(),
            property_settings: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct EvidenceLine {
    file: String,
    line: usize,
    key: Option<String>,
    text: String,
}

pub fn run(command: ObsCommand) -> Result<i32> {
    match command {
        ObsCommand::ExportToNix(args) => {
            export_to_nix(args)?;
            Ok(0)
        }
        ObsCommand::PluginInspect(args) => plugin_inspect_command(args),
    }
}

fn export_to_nix(args: ExportToNix) -> Result<()> {
    let root = args.config_dir.unwrap_or_else(default_config_dir);

    println!("{{");

    let global_ini = root.join("global.ini");
    if global_ini.exists() {
        emit_assignment(
            &["settings", "global"],
            ini_to_json(&global_ini, &args.redaction)?,
        )?;
    }

    let user_ini = root.join("user.ini");
    if user_ini.exists() {
        emit_assignment(
            &["settings", "user"],
            ini_to_json(&user_ini, &args.redaction)?,
        )?;
    }

    let profiles = root.join("basic/profiles");
    if profiles.exists() {
        for profile in sorted_dirs(&profiles)? {
            let profile_name = path_name(&profile)?;
            let basic_ini = profile.join("basic.ini");
            if basic_ini.exists() {
                emit_assignment(
                    &["profiles", &profile_name, "settings"],
                    ini_to_json(&basic_ini, &args.redaction)?,
                )?;
            }
            for (file_name, option) in [
                ("streamEncoder.json", "streamEncoder"),
                ("recordEncoder.json", "recordEncoder"),
            ] {
                let file = profile.join(file_name);
                if file.exists() {
                    emit_assignment(
                        &["profiles", &profile_name, option],
                        sanitized_json_file(&file, &args.redaction)?,
                    )?;
                }
            }
        }
    }

    let scenes = root.join("basic/scenes");
    if scenes.exists() {
        for scene in sorted_files_with_ext(&scenes, "json")? {
            let name = scene
                .file_stem()
                .and_then(OsStr::to_str)
                .ok_or_else(|| miette!("invalid scene filename `{}`", scene.display()))?;
            emit_assignment(
                &["sceneCollections", name, "raw"],
                sanitized_json_file(&scene, &args.redaction)?,
            )?;
        }
    }

    let plugin_config = root.join("plugin_config");
    if plugin_config.exists() {
        for file in sorted_recursive_files(&plugin_config)? {
            if file.extension().and_then(OsStr::to_str) == Some("json") {
                let rel = file
                    .strip_prefix(&plugin_config)
                    .into_diagnostic()?
                    .to_string_lossy()
                    .replace('\\', "/");
                let text =
                    serde_json::to_string_pretty(&sanitized_json_file(&file, &args.redaction)?)
                        .into_diagnostic()?
                        + "\n";
                emit_assignment(&["extraConfigFiles", &rel, "text"], Value::String(text))?;
            }
        }
    }

    println!("}}");
    Ok(())
}

fn plugin_inspect_command(args: PluginInspect) -> Result<i32> {
    match args.command {
        Some(PluginInspectCommand::Verify(verify)) => {
            plugin_inspect_verify(verify)?;
            Ok(0)
        }
        None => {
            let root = exactly_one_dir(args.source_dir, args.install_dir, "plugin-inspect")?;
            let evidence = inspect_plugin(&root)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&evidence).into_diagnostic()?
            );
            Ok(0)
        }
    }
}

fn plugin_inspect_verify(args: PluginInspectVerify) -> Result<()> {
    let root = exactly_one_dir(args.source_dir, args.install_dir, "plugin-inspect verify")?;
    let expected: PluginEvidence = serde_json::from_reader(
        fs::File::open(&args.evidence)
            .into_diagnostic()
            .wrap_err_with(|| format!("cannot open evidence file {}", args.evidence.display()))?,
    )
    .into_diagnostic()?;
    let actual = inspect_plugin(&root)?;
    if actual == expected {
        println!("plugin evidence ok");
        Ok(())
    } else {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&actual).into_diagnostic()?
        );
        bail!("plugin evidence mismatch");
    }
}

fn exactly_one_dir(
    source_dir: Option<PathBuf>,
    install_dir: Option<PathBuf>,
    command: &str,
) -> Result<PathBuf> {
    match (source_dir, install_dir) {
        (Some(path), None) | (None, Some(path)) => Ok(path),
        (None, None) => {
            bail!("{command} requires --source-dir or --install-dir");
        }
        (Some(_), Some(_)) => {
            bail!("{command} accepts only one of --source-dir or --install-dir");
        }
    }
}

fn inspect_plugin(root: &Path) -> Result<PluginEvidence> {
    let mut evidence = PluginEvidence::empty();
    for file in sorted_recursive_files(root)? {
        if !is_text_candidate(&file) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .to_string();
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let item = EvidenceLine {
                file: rel.clone(),
                line: idx + 1,
                key: quoted_key(trimmed),
                text: trimmed.to_string(),
            };
            if trimmed.contains("obs_source_info") {
                evidence.source_ids.push(item);
            } else if trimmed.contains("obs_filter_info") {
                evidence.filter_ids.push(item);
            } else if trimmed.contains("obs_output_info") {
                evidence.output_ids.push(item);
            } else if trimmed.contains("obs_encoder_info") {
                evidence.encoder_ids.push(item);
            } else if trimmed.contains("obs_register_") {
                evidence.registrations.push(item);
            } else if trimmed.contains("obs_data_set_default_") {
                evidence.setting_defaults.push(item);
            } else if trimmed.contains("obs_properties_add_") {
                evidence.property_settings.push(item);
            }
        }
    }
    Ok(evidence)
}

fn quoted_key(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn emit_assignment(path: &[&str], value: Value) -> Result<()> {
    let lhs = format!(
        "programs.obs-studio.{}",
        path.iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .into_diagnostic()?
            .join(".")
    );
    println!("  {lhs} = {};", to_nix(&value, 2)?);
    Ok(())
}

fn default_config_dir() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("obs-studio")
}

fn is_text_candidate(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("c" | "cc" | "cpp" | "cxx" | "h" | "hpp" | "hh" | "m" | "mm" | "json" | "ini")
    )
}
