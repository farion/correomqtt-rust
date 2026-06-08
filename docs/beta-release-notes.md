# CorreoMQTT Rust Beta Release Notes

Status: beta notes for the Rust desktop port, version 0.1.0.

These notes cover the unsigned beta packages produced by the Rust workspace.
The M8 beta is internal only: it is intended for engineering, QA, and invited
company beta validation of migration and core workflows before any signed,
notarized, public, or store-distributed release.

No public beta distribution, signing, notarization, auto-update service, paid CI
capacity, certificate purchase, credential change, or external release
commitment is approved for this milestone. Any future signed, notarized, or
public beta path requires a separate CEO or board approval request before
engineering introduces secrets, certificates, paid services, or release
commitments.

## Supported Unsigned Packages

Unsigned beta archives are produced with:

```bash
cargo xtask package
```

The supported beta package layouts are:

| Platform | Target triple | Archive contents |
| --- | --- | --- |
| Windows | `x86_64-pc-windows-msvc` | `CorreoMQTT/correomqtt.exe`, icon, metadata JSON |
| macOS | `aarch64-apple-darwin` | `CorreoMQTT.app` bundle with `Info.plist`, executable, and ICNS icon |
| Linux | `x86_64-unknown-linux-gnu` | relocatable `CorreoMQTT/` tree with binary, desktop entry, icon, and AppStream metadata |

Artifacts are written under `dist/beta/` by default:

```text
CorreoMQTT-0.1.0-beta-<target-triple>.zip
CorreoMQTT-0.1.0-beta-<target-triple>.zip.sha256
SHA256SUMS
```

The archives are intentionally unsigned. Code signing, notarization, installers,
auto-update, and external release publishing are outside this internal beta
package.

## Migration And Rollback

The Rust beta detects legacy Java data in the existing CorreoMQTT locations:

| OS | Legacy path |
| --- | --- |
| Windows | `%APPDATA%/CorreoMqtt` |
| macOS | `~/Library/Application Support/CorreoMqtt` |
| Linux and Unix-like systems | `~/.correomqtt` |

The migration path preserves known `config.json` fields, histories, scripts,
script execution metadata, script logs, and `.cqc` connection imports. Unknown
or unmappable JSON fields are ignored and reported as migration warnings.

Legacy password import supports both known Java password file formats:

- `AES/GCM/NoPadding`
- `AES/CBC/PKCS5Padding`

New secrets are stored through the OS keyring abstraction after migration. The
beta release notes, logs, fixtures, and diagnostics must not include passwords,
key material, decrypted password maps, or export passwords.

Before replacing or modifying user data, the migration flow must create a
timestamped backup. If migration has to be rolled back, stop the Rust beta,
restore the backed-up legacy directory, and reopen the last Java release against
that restored data. Keep the beta archive and the Java release data separate
while validating migrated profiles.

## Plugin Compatibility

Java plugin compatibility is intentionally dropped in the Rust port.

The Rust beta does not load PF4J jars and does not treat old `plugins/jars`,
old `plugins/config`, old `protocol.xml`, or PF4J metadata as compatibility
targets. Plugin state is reinitialized from Rust plugin manifests and bundled
Rust or WASM replacements.

The initial plugin surface is limited to non-UI extension points:

- incoming message transforms
- outgoing message transforms
- message validators
- detail byte transforms
- detail formatters

Arbitrary JavaFX or egui UI injection is not part of this beta.

## Diagnostics

User-visible diagnostics are available in the app diagnostics strip and the
Diagnostics workspace. Migration warnings, MQTT connection failures, plugin
failures, script failures, and persistence failures should be visible there
without blocking normal workflows.

Script execution logs are stored incrementally under the current profile data
directory:

```text
scripts/logs/<script-relative-path>/<execution-id>.log
scripts/executions/<script-relative-path>/<execution-id>.json
```

For developer and QA runs, `CORREOMQTT_CONFIG_DIR` can be used as the profile
data directory. Packaged builds otherwise use the platform data directory chosen
by the app.

App-level tracing is initialized by the Rust desktop shell and can be filtered
with the standard tracing environment filter. A dedicated packaged app log file
is not a beta compatibility guarantee yet. Packaged beta builds should keep
diagnostics redacted and should not require users to share secrets or private
connection material when reporting failures.

## Known Limitations

- Packages are unsigned and may trigger Windows SmartScreen, macOS Gatekeeper,
  or Linux desktop trust warnings. QA should treat these trust warnings as
  expected for M8 unsigned artifacts and should still record unexpected install
  failures or missing runtime notes.
- The package command stages native layouts, but each OS package should be
  produced on a matching runner unless a cross-compilation toolchain is
  explicitly configured.
- Legacy Java/PF4J plugins are not migrated or loaded.
- JavaFX plugin UI extensions have no Rust beta equivalent.
- Advanced GraalJS scripts that depended on arbitrary Java host access may need
  updates for the Rust scripting host.
- TLS keystore formats that do not map cleanly to Rust TLS are preserved as
  configuration metadata and should produce diagnostics instead of being silently
  dropped.
- Old plugin hook configuration is reported as ignored and replaced by the Rust
  plugin manifest model.
- Signing, notarization, installers, auto-update, paid CI capacity, certificate
  purchase, credential changes, and public release publishing remain future
  release work that requires CEO or board approval before implementation.

## Verification Notes

For beta signoff, validate:

- fresh install startup on Windows, macOS, and Linux
- migrated install startup from each legacy data path
- `.cqc` import/export for plain and encrypted connection exports
- message import/export compatibility
- script execution logs and cancellation
- built-in replacement plugins for transforms, validators, and formatters
- diagnostics redaction for passwords, key material, and export passwords
