use std::path::{Path, PathBuf};

use crate::plugin_repository::stage_local_plugins;
use crate::XtaskError;

use super::{Platform, APP_NAME};

pub(super) fn stage(platform: Platform, stage_dir: &Path) -> Result<(), XtaskError> {
    let executable_dir = executable_dir(platform, stage_dir);
    stage_local_plugins(&executable_dir)
}

fn executable_dir(platform: Platform, stage_dir: &Path) -> PathBuf {
    match platform {
        Platform::Linux => stage_dir.join(APP_NAME).join("bin"),
        Platform::Macos => stage_dir
            .join(format!("{APP_NAME}.app"))
            .join("Contents/MacOS"),
        Platform::Windows => stage_dir.join(APP_NAME),
    }
}
