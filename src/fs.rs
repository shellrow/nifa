use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, io::Write, path::Path};

use crate::cli::OutputFormat;

pub fn export<T: Serialize>(
    format: OutputFormat,
    output: Option<&Path>,
    data: &T,
) -> Result<()> {
    let (bytes, ext_default) = match format {
        OutputFormat::Json => (serde_json::to_vec_pretty(data)?, "json"),
        OutputFormat::Yaml => (serde_yaml::to_string(data)?.into_bytes(), "yaml"),
        other => {
            tracing::warn!(
                "note: --export with {:?} is not supported; falling back to JSON.",
                other
            );
            (serde_json::to_vec_pretty(data)?, "json")
        }
    };

    if let Some(path) = output {
        atomic_write(path, &bytes, ext_default)?;
        tracing::info!(
            "Exported {} bytes to {}",
            bytes.len(),
            path.display()
        );
    } else {
        std::io::stdout().write_all(&bytes).context("write stdout")?;
    }

    Ok(())
}

fn atomic_write(path: &Path, data: &[u8], ext_default: &str) -> Result<()> {
    let target = if path.extension().is_none() {
        path.with_extension(ext_default)
    } else {
        path.to_path_buf()
    };
    let tmp = target.with_extension("tmp");
    fs::write(&tmp, data).with_context(|| format!("write temp {}", tmp.display()))?;
    fs::rename(&tmp, &target).with_context(|| format!("rename to {}", target.display()))?;
    Ok(())
}
