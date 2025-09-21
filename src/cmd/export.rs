use std::{fs, io::Write, path::Path};

use crate::cli::{Cli, ExportArgs, OutputFormat};
use anyhow::{Context, Result};

pub fn export_snapshot(cli: &Cli, args: &ExportArgs) -> Result<()> {
    let snapshot = crate::collector::collect_snapshot()?;
    let (bytes, ext_default) = match cli.format {
        OutputFormat::Json | OutputFormat::Tree => {
            // tree are ignored for export, default to json
            (serde_json::to_vec_pretty(&snapshot)?, "json")
        }
        OutputFormat::Yaml => (serde_yaml::to_string(&snapshot)?.into_bytes(), "yaml"),
    };
    if let Some(path) = &args.output {
        atomic_write(path, &bytes, ext_default)?;
        eprintln!("Exported {} bytes to {}", bytes.len(), path.display());
    } else {
        // if no output file, write to stdout
        std::io::stdout().write_all(&bytes).context("write stdout")?;
    }
    Ok(())
}

/// Atomically write data to a file (with default extension if missing)
fn atomic_write(path: &Path, data: &[u8], ext_default: &str) -> Result<()> {
    // Add default extension if missing
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
