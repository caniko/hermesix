use clap::{Args, ValueEnum};
use miette::{bail, Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

const SENSITIVE_PARTS: &[&str] = &["token", "key", "password", "secret", "auth", "cookie"];
const RUNTIME_KEYS: &[&str] = &[
    "geometry",
    "DockState",
    "LastVersion",
    "InfoIncrement",
    "CookieId",
    "LastUpdateCheck",
];
const LOCAL_PATH_PARTS: &[&str] = &["path", "dir", "directory", "file"];

#[derive(Debug, Args)]
pub struct Redact {
    pub input: std::path::PathBuf,

    #[arg(long, value_enum, default_value_t = RedactFormat::Auto)]
    pub format: RedactFormat,

    #[command(flatten)]
    pub redaction: Redaction,
}

#[derive(Debug, Args, Default, Clone)]
pub struct Redaction {
    #[arg(long)]
    pub include_sensitive: bool,

    #[arg(long)]
    pub include_runtime: bool,

    #[arg(long)]
    pub include_local_paths: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RedactFormat {
    Auto,
    Json,
    Ini,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Ini,
    Json,
    Raw,
}

pub type Ini = BTreeMap<String, BTreeMap<String, String>>;

pub fn redact_command(args: Redact) -> Result<()> {
    let kind = detect_kind(&args.input, args.format);
    match kind {
        FileKind::Json => println!(
            "{}",
            serde_json::to_string_pretty(&sanitized_json_file(&args.input, &args.redaction)?)
                .into_diagnostic()?
        ),
        FileKind::Ini => print!(
            "{}",
            render_ini(&ini_to_sections(&args.input, &args.redaction)?)
        ),
        FileKind::Raw => {
            let mut text = String::new();
            fs::File::open(args.input)
                .into_diagnostic()?
                .read_to_string(&mut text)
                .into_diagnostic()?;
            print!("{text}");
        }
    }
    Ok(())
}

pub fn parse_by_kind(path: &Path, rel_path: &str, kind: FileKind) -> Result<()> {
    match effective_kind(rel_path, kind) {
        FileKind::Json => {
            let _: Value = serde_json::from_reader(
                fs::File::open(path)
                    .into_diagnostic()
                    .wrap_err_with(|| format!("cannot open {}", path.display()))?,
            )
            .into_diagnostic()?;
        }
        FileKind::Ini => {
            let _ = parse_ini(path)?;
        }
        FileKind::Raw => {}
    }
    Ok(())
}

pub fn portable_policy_check(
    path: &Path,
    rel_path: &str,
    kind: FileKind,
    redaction: &Redaction,
) -> Result<()> {
    match effective_kind(rel_path, kind) {
        FileKind::Json => {
            let original: Value = serde_json::from_reader(
                fs::File::open(path)
                    .into_diagnostic()
                    .wrap_err_with(|| format!("cannot open {}", path.display()))?,
            )
            .into_diagnostic()?;
            let sanitized = sanitize_json(original.clone(), redaction, "");
            if sanitized != original {
                bail!("contains non-portable or sensitive JSON fields");
            }
        }
        FileKind::Ini => {
            let original = parse_ini(path)?;
            let sanitized = sanitize_ini(original.clone(), redaction);
            if sanitized != original {
                bail!("contains non-portable or sensitive INI fields");
            }
        }
        FileKind::Raw => {}
    }
    Ok(())
}

pub fn kind_name(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Ini => "ini",
        FileKind::Json => "json",
        FileKind::Raw => "raw",
    }
}

pub fn detect_kind(path: &Path, format: RedactFormat) -> FileKind {
    match format {
        RedactFormat::Json => FileKind::Json,
        RedactFormat::Ini => FileKind::Ini,
        RedactFormat::Auto => match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("json") => FileKind::Json,
            Some("ini") => FileKind::Ini,
            _ => FileKind::Raw,
        },
    }
}

pub fn sanitized_json_file(path: &Path, redaction: &Redaction) -> Result<Value> {
    let value: Value = serde_json::from_reader(
        fs::File::open(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("cannot open {}", path.display()))?,
    )
    .into_diagnostic()?;
    Ok(sanitize_json(value, redaction, ""))
}

pub fn ini_to_json(path: &Path, redaction: &Redaction) -> Result<Value> {
    let sections = ini_to_sections(path, redaction)?;
    let mut root = Map::new();
    for (section, values) in sections {
        let mut object = Map::new();
        for (key, value) in values {
            object.insert(key, Value::String(value));
        }
        root.insert(section, Value::Object(object));
    }
    Ok(Value::Object(root))
}

pub fn ini_to_sections(path: &Path, redaction: &Redaction) -> Result<Ini> {
    Ok(sanitize_ini(parse_ini(path)?, redaction))
}

pub fn render_ini(ini: &Ini) -> String {
    let mut out = String::new();
    for (section, values) in ini {
        out.push('[');
        out.push_str(section);
        out.push_str("]\n");
        for (key, value) in values {
            out.push_str(key);
            out.push('=');
            out.push_str(value);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

fn effective_kind(rel_path: &str, kind: FileKind) -> FileKind {
    if kind != FileKind::Raw {
        return kind;
    }
    if rel_path.ends_with(".json") {
        FileKind::Json
    } else if rel_path.ends_with(".ini") {
        FileKind::Ini
    } else {
        FileKind::Raw
    }
}

fn sanitize_json(value: Value, redaction: &Redaction, key: &str) -> Value {
    if should_omit(key, &value, redaction) {
        return Value::Null;
    }
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter_map(|(child_key, child_value)| {
                    if should_omit(&child_key, &child_value, redaction) {
                        None
                    } else {
                        Some((
                            child_key.clone(),
                            sanitize_json(child_value, redaction, &child_key),
                        ))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| sanitize_json(item, redaction, key))
                .filter(|item| !item.is_null())
                .collect(),
        ),
        other => other,
    }
}

fn should_omit(key: &str, value: &Value, redaction: &Redaction) -> bool {
    let lowered = key.to_ascii_lowercase();
    if !redaction.include_sensitive && SENSITIVE_PARTS.iter().any(|part| lowered.contains(part)) {
        return true;
    }
    if !redaction.include_runtime && RUNTIME_KEYS.contains(&key) {
        return true;
    }
    if !redaction.include_local_paths && value.as_str().is_some() {
        let pathish = LOCAL_PATH_PARTS
            .iter()
            .any(|part| lowered.ends_with(part) || lowered.contains(part));
        if pathish {
            let string = value.as_str().unwrap_or_default();
            return string.starts_with('/') || string.starts_with('~');
        }
    }
    false
}

fn parse_ini(path: &Path) -> Result<Ini> {
    let text = fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot read {}", path.display()))?;
    let mut sections: Ini = BTreeMap::new();
    let mut current = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current = trimmed[1..trimmed.len() - 1].to_string();
            sections.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(sections)
}

fn sanitize_ini(ini: Ini, redaction: &Redaction) -> Ini {
    ini.into_iter()
        .filter_map(|(section, values)| {
            let values: BTreeMap<_, _> = values
                .into_iter()
                .filter(|(key, value)| !should_omit(key, &Value::String(value.clone()), redaction))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some((section, values))
            }
        })
        .collect()
}
