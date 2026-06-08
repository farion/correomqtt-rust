use super::checksums::{hex_digest, sha256_file, write_checksum_files};
use super::*;
use sha2::{Digest, Sha256};

#[test]
fn detects_supported_platforms_from_targets() {
    assert_eq!(
        Platform::from_target("x86_64-pc-windows-msvc").unwrap(),
        Platform::Windows
    );
    assert_eq!(
        Platform::from_target("aarch64-apple-darwin").unwrap(),
        Platform::Macos
    );
    assert_eq!(
        Platform::from_target("x86_64-unknown-linux-gnu").unwrap(),
        Platform::Linux
    );
}

#[test]
fn package_names_are_predictable() {
    let plan = PackagePlan::new(
        "x86_64-unknown-linux-gnu".to_owned(),
        PathBuf::from("dist/beta"),
    );
    assert_eq!(
        plan.artifact_file_name(),
        format!(
            "CorreoMQTT-{}-beta-x86_64-unknown-linux-gnu.zip",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn sha256_hex_is_lowercase() {
    let digest = Sha256::digest(b"abc");
    assert_eq!(
        hex_digest(&digest),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn zip_entries_have_stable_metadata_and_modes() {
    let root = temp_package_root("zip-metadata");
    let source = root.join("stage");
    write_file(&source.join("bin").join(BIN_NAME), b"binary").unwrap();
    write_file(&source.join("README.txt"), b"readme").unwrap();

    let first = root.join("first.zip");
    let second = root.join("second.zip");
    zip_dir(&source, &first).unwrap();
    zip_dir(&source, &second).unwrap();

    assert_eq!(sha256_file(&first).unwrap(), sha256_file(&second).unwrap());

    let file = std::fs::File::open(&first).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let file = archive.by_index(index).unwrap();
        entries.push((
            file.name().to_owned(),
            file.last_modified().unwrap(),
            file.unix_mode().unwrap() & 0o777,
        ));
    }
    let names = entries
        .iter()
        .map(|entry| entry.0.clone())
        .collect::<Vec<_>>();
    let mut sorted_names = names.clone();
    sorted_names.sort();

    assert_eq!(names, sorted_names);
    assert_eq!(
        entries,
        vec![
            ("stage/README.txt".to_owned(), DateTime::default(), 0o644),
            ("stage/bin/".to_owned(), DateTime::default(), 0o755),
            (
                "stage/bin/correomqtt".to_owned(),
                DateTime::default(),
                0o755
            ),
        ]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_smoke_rejects_unexpected_zip_outputs() {
    let root = temp_package_root("package-guard");
    let out_dir = root.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let plan = PackagePlan::new("x86_64-unknown-linux-gnu".to_owned(), out_dir.clone());
    let artifact = plan.artifact_path();
    write_file(&artifact, b"package").unwrap();
    let checksum = write_checksum_files(&artifact, &out_dir).unwrap();

    write_file(&out_dir.join("stale.zip"), b"stale").unwrap();
    let output = Some(PackageOutput {
        command: "cargo xtask package-smoke --target x86_64-unknown-linux-gnu".to_owned(),
        target: plan.target,
        out_dir,
        artifact,
        checksum,
    });

    let error = guard::verify(&output).unwrap_err().to_string();
    assert!(error.contains("unexpected ZIP outputs"));
    std::fs::remove_dir_all(root).unwrap();
}

fn temp_package_root(name: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("{name}-{}-{timestamp}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    root
}
