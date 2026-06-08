use std::fs;
use std::path::{Path, PathBuf};

use crate::XtaskError;

use super::checksums::{artifact_file_name, sha256_file};
use super::PackageOutput;

pub(super) fn verify(output: &Option<PackageOutput>) -> Result<(), XtaskError> {
    let Some(output) = output else {
        return Ok(());
    };

    let file_name = artifact_file_name(&output.artifact)?;
    let expected_line = format!("{}  {file_name}\n", output.checksum);
    let checksum_path = output.out_dir.join(format!("{file_name}.sha256"));
    let sums_path = output.out_dir.join("SHA256SUMS");

    ensure(
        output,
        output.artifact.exists(),
        format!("missing expected archive {}", output.artifact.display()),
    )?;
    ensure(
        output,
        checksum_path.exists(),
        format!("missing per-archive checksum {}", checksum_path.display()),
    )?;
    ensure(
        output,
        sums_path.exists(),
        format!("missing checksum summary {}", sums_path.display()),
    )?;

    let actual_checksum = sha256_file(&output.artifact)?;
    ensure(
        output,
        actual_checksum == output.checksum,
        format!(
            "archive checksum mismatch for {}: expected {}, actual {}",
            output.artifact.display(),
            output.checksum,
            actual_checksum
        ),
    )?;

    let actual_sha = fs::read_to_string(&checksum_path)?;
    ensure(
        output,
        actual_sha == expected_line,
        format!(
            "{} did not match expected `{}`",
            checksum_path.display(),
            expected_line.trim_end()
        ),
    )?;

    let actual_sums = fs::read_to_string(&sums_path)?;
    ensure(
        output,
        actual_sums == expected_line,
        format!(
            "{} did not contain exactly the expected package checksum",
            sums_path.display()
        ),
    )?;

    let zip_files = files_with_extension(&output.out_dir, "zip")?;
    ensure(
        output,
        zip_files == vec![output.artifact.clone()],
        format!(
            "unexpected ZIP outputs in {}: expected [{}], actual [{}]",
            output.out_dir.display(),
            output.artifact.display(),
            display_paths(&zip_files)
        ),
    )?;

    let sha_files = files_with_suffix(&output.out_dir, ".zip.sha256")?;
    ensure(
        output,
        sha_files == vec![checksum_path.clone()],
        format!(
            "unexpected per-archive checksum outputs in {}: expected [{}], actual [{}]",
            output.out_dir.display(),
            checksum_path.display(),
            display_paths(&sha_files)
        ),
    )?;

    println!("package-smoke: target {}", output.target);
    println!("package-smoke: command {}", output.command);
    println!("package-smoke: artifact {}", output.artifact.display());
    println!("package-smoke: sha256 {}", output.checksum);
    println!("package-smoke: verified {}", checksum_path.display());
    println!("package-smoke: verified {}", sums_path.display());
    Ok(())
}

fn ensure(output: &PackageOutput, condition: bool, message: String) -> Result<(), XtaskError> {
    if condition {
        Ok(())
    } else {
        Err(XtaskError::PackageGuard {
            target: output.target.clone(),
            command: output.command.clone(),
            artifact: output.artifact.display().to_string(),
            message,
        })
    }
}

fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, XtaskError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == extension)
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn files_with_suffix(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, XtaskError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with(suffix))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
