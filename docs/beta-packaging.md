# Unsigned Beta Packaging

The Rust port uses `xtask` for internal unsigned beta packaging:

```bash
cargo xtask package
```

By default this builds `correo-app` in release mode for the current host and
writes artifacts under `dist/beta/`. These artifacts are for internal
engineering, QA, and invited company beta validation only.

Useful options:

```bash
cargo xtask package --target x86_64-pc-windows-msvc
cargo xtask package --target aarch64-apple-darwin
cargo xtask package --target x86_64-unknown-linux-gnu
cargo xtask package --out-dir dist/beta-local
cargo xtask package --no-build
cargo xtask package-smoke --target x86_64-unknown-linux-gnu --out-dir dist/beta/x86_64-unknown-linux-gnu
```

`--target` selects the package layout and passes the same target triple to
`cargo build`. Each OS runner should package its native target unless a
cross-compilation toolchain is explicitly configured.

`cargo xtask package-smoke` runs the same package build and then verifies the
artifact guardrails used by CI: exactly one ZIP in the output directory, the
expected predictable archive name, a matching per-archive `.zip.sha256`, and a
`SHA256SUMS` file that contains exactly that archive checksum. Use a
target-specific output directory for this command so stale packages from other
targets fail clearly instead of being mixed into the smoke result.

Artifacts use predictable names:

```text
CorreoMQTT-<version>-beta-<target-triple>.zip
CorreoMQTT-<version>-beta-<target-triple>.zip.sha256
SHA256SUMS
```

The archives are intentionally unsigned and may trigger OS trust warnings such
as Windows SmartScreen, macOS Gatekeeper, or Linux desktop trust prompts. QA
should treat those warnings as expected for M8 unsigned artifacts while still
recording unexpected install failures or missing runtime notes.

No public beta distribution, signing, notarization, auto-update service, paid CI
capacity, certificate purchase, credential change, or external release
commitment is approved for M8. Any future signed, notarized, or public beta path
requires a separate CEO or board approval request before engineering introduces
secrets, certificates, paid services, or release commitments.

Reproducibility smoke for a host that already has `target/release/correomqtt`:

```bash
out_a=$(mktemp -d)
out_b=$(mktemp -d)
cargo xtask package --no-build --out-dir "$out_a"
cargo xtask package --no-build --out-dir "$out_b"
cmp "$out_a"/CorreoMQTT-*-beta-*.zip "$out_b"/CorreoMQTT-*-beta-*.zip
cat "$out_a"/CorreoMQTT-*-beta-*.zip.sha256
cat "$out_b"/CorreoMQTT-*-beta-*.zip.sha256
```

The ZIP entries are written in sorted order with fixed entry timestamps,
expected executable modes for app binaries, `0644` modes for non-executables,
predictable artifact names, per-archive `.zip.sha256`, and a sorted
`SHA256SUMS`.

CI packaging smoke:

| Platform | Runner label | Target triple | Local parity command |
| --- | --- | --- | --- |
| Linux | `ubuntu-24.04` | `x86_64-unknown-linux-gnu` | `cargo xtask package-smoke --target x86_64-unknown-linux-gnu --out-dir dist/beta/x86_64-unknown-linux-gnu` |
| Windows | `windows-2025` | `x86_64-pc-windows-msvc` | `cargo xtask package-smoke --target x86_64-pc-windows-msvc --out-dir dist/beta/x86_64-pc-windows-msvc` |
| macOS | `macos-15` | `aarch64-apple-darwin` | `cargo xtask package-smoke --target aarch64-apple-darwin --out-dir dist/beta/aarch64-apple-darwin` |

The workflow uploads only the ZIP, the matching `.zip.sha256`, and
`SHA256SUMS` after `package-smoke` passes. CI output records the commit, runner,
target, command, artifact path, and SHA-256. Current runner gaps are Linux ARM,
Windows ARM, and macOS x86_64 package coverage; adding those requires a CTO
runner/target decision and, for non-native builds, a documented
cross-compilation toolchain. Signing, notarization, installers, paid CI
capacity, credential changes, auto-update, and external release publishing are
not part of this smoke workflow.

Platform layouts:

- Windows: `CorreoMQTT/correomqtt.exe`, `icons/Icon.ico`, metadata JSON.
- macOS: `CorreoMQTT.app` with `Info.plist`, `PkgInfo`, executable, and ICNS.
- Linux: relocatable `CorreoMQTT/` tree with binary, desktop entry, icon, and
  AppStream metadata.

Runtime data notes:

- `CORREOMQTT_CONFIG_DIR` overrides the runtime root for config, histories, and
  script sidecar logs/metadata. Use a synthetic empty directory for QA runs that
  must not touch a developer profile.
- Without `CORREOMQTT_CONFIG_DIR`, the Rust beta uses
  `ProjectDirs::from("org", "CorreoMQTT", "CorreoMQTT").data_dir()`: Linux
  `$XDG_DATA_HOME/correomqtt` or `~/.local/share/correomqtt`, macOS
  `~/Library/Application Support/org.CorreoMQTT.CorreoMQTT`, and Windows
  `%APPDATA%\\CorreoMQTT\\CorreoMQTT\\data`.
- Current config and per-connection history files live under that root. Script
  execution metadata and logs use `scripts/executions/` and `scripts/logs/`
  below the same root when scripting persistence writes them.
- App diagnostics are currently emitted to stdout/stderr through tracing and
  controlled with `RUST_LOG`; the package does not create a separate app log
  directory yet.
- Startup also checks legacy Java roots for migration: `%APPDATA%/CorreoMqtt`,
  `~/Library/Application Support/CorreoMqtt`, and `~/.correomqtt`.
