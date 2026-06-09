use std::fs;
use std::path::{Path, PathBuf};

use correo_plugins::{PluginManifest, PluginRepositoryDefinition, PluginRepositoryEntry};

use crate::XtaskError;

pub(crate) const LOCAL_PLUGIN_REPOSITORY_FILE: &str = "repository.json";
const DEFAULT_REPOSITORY_ID: &str = "local-bundled-rust";
const DEFAULT_REPOSITORY_NAME: &str = "Bundled Rust Plugins";
const SAVE_MANIPULATOR_MANIFEST: &str = include_str!("../../plugins/save-manipulator/plugin.toml");
const SAVE_MANIPULATOR_PACKAGE_PATH: &str = "plugins/save-manipulator";

pub(crate) fn run(args: Vec<String>) -> Result<(), XtaskError> {
    let config = PluginRepositoryConfig::from_args(args)?;
    if config.show_help {
        print_help();
        return Ok(());
    }

    let path = config.out_dir.join(LOCAL_PLUGIN_REPOSITORY_FILE);
    write_bundled_repository(&path)?;
    println!("plugin-repository: {}", path.display());
    Ok(())
}

pub(crate) fn write_bundled_repository(path: &Path) -> Result<(), XtaskError> {
    let mut repository = PluginRepositoryDefinition::from_bundled_plugins(
        DEFAULT_REPOSITORY_ID,
        DEFAULT_REPOSITORY_NAME,
    );
    repository.plugins.push(save_manipulator_entry()?);
    repository
        .plugins
        .sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    repository.validate()?;
    let mut json = serde_json::to_vec_pretty(&repository)?;
    json.push(b'\n');
    write_file(path, &json)
}

fn save_manipulator_entry() -> Result<PluginRepositoryEntry, XtaskError> {
    let manifest = PluginManifest::from_toml_str(SAVE_MANIPULATOR_MANIFEST)
        .map_err(correo_plugins::PluginRepositoryError::from)?;
    PluginRepositoryEntry::local_package(manifest, SAVE_MANIPULATOR_PACKAGE_PATH)
        .map_err(XtaskError::from)
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), XtaskError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn print_help() {
    println!("Usage: cargo xtask plugin-repository [--out-dir <dir>]");
    println!();
    println!("Writes the local Rust plugin repository definition.");
}

#[derive(Debug)]
struct PluginRepositoryConfig {
    out_dir: PathBuf,
    show_help: bool,
}

impl PluginRepositoryConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut out_dir = PathBuf::from("dist/plugins");
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-repository option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self { out_dir, show_help })
    }
}
