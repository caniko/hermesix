use miette::{IntoDiagnostic, Result};
use serde_json::Value;

pub fn to_nix(value: &Value, indent: usize) -> Result<String> {
    let pad = " ".repeat(indent);
    let next = " ".repeat(indent + 2);
    Ok(match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(value).into_diagnostic()?,
        Value::Array(items) if items.is_empty() => "[]".to_string(),
        Value::Array(items) => {
            let rendered = items
                .iter()
                .map(|item| Ok(format!("{next}{}", to_nix(item, indent + 2)?)))
                .collect::<Result<Vec<_>>>()?
                .join("\n");
            format!("[\n{rendered}\n{pad}]")
        }
        Value::Object(map) if map.is_empty() => "{}".to_string(),
        Value::Object(map) => {
            let rendered = map
                .iter()
                .map(|(key, child)| {
                    Ok(format!(
                        "{next}{} = {};",
                        serde_json::to_string(key).into_diagnostic()?,
                        to_nix(child, indent + 2)?
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join("\n");
            format!("{{\n{rendered}\n{pad}}}")
        }
    })
}
