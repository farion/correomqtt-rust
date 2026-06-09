use std::path::{Path, PathBuf};

use crate::plugin_repository::{write_bundled_repository, LOCAL_PLUGIN_REPOSITORY_FILE};
use crate::XtaskError;

use super::{Platform, APP_NAME};

pub(super) fn stage(platform: Platform, stage_dir: &Path) -> Result<(), XtaskError> {
    write_bundled_repository(&repository_path(platform, stage_dir))
}

pub(super) fn repository_path(platform: Platform, stage_dir: &Path) -> PathBuf {
    match platform {
        Platform::Linux => stage_dir
            .join(APP_NAME)
            .join("share/correomqtt/plugins")
            .join(LOCAL_PLUGIN_REPOSITORY_FILE),
        Platform::Macos => stage_dir
            .join(format!("{APP_NAME}.app"))
            .join("Contents/Resources/plugins")
            .join(LOCAL_PLUGIN_REPOSITORY_FILE),
        Platform::Windows => stage_dir
            .join(APP_NAME)
            .join("plugins")
            .join(LOCAL_PLUGIN_REPOSITORY_FILE),
    }
}
