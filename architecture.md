# CorreoMQTT Rust Port Architecture

Status: draft strategy for the Rust port.

## Goals

- Port CorreoMQTT from the current Java 17, JavaFX, HiveMQ, PF4J, GraalJS stack to a native Rust desktop application.
- Use `egui` for the desktop UI.
- Preserve user value first: connections, secrets, histories, scripts, imports, exports, and common plugin behavior.
- Keep the rewrite testable by isolating MQTT, storage, scripting, plugins, and UI behind small boundaries.
- Keep every Rust source file under 500 lines. Split files by responsibility before they grow past that limit.

## Current System Summary

The current application is a Maven multi-module Java app:

- `core`: MQTT clients, connection lifecycle, pub/sub tasks, JSON persistence, imports/exports, scripting, plugin contracts, keyrings, logging, and utilities.
- `gui`: JavaFX/FXML views for connections, publish/subscribe, scripting, plugins, import/export, settings, logs, and onboarding.
- `plugins`: bundled PF4J plugins for base64, JSON/XML formatting, validators, detail manipulators, save, system topics, XSD validation, and zip operations.
- `di`: custom annotation-based dependency injection and event bus.

Important persisted user data:

- `config.json`: connections, themes, settings, UI state, plugin repositories.
- `passwords.json`: encrypted connection secrets.
- `hooks.json`: plugin hook configuration.
- Per-connection histories: `*_publishHistory.json`, `*_publishMessageHistory.json`, `*_subscriptionHistory.json`.
- `scripts/`: user JavaScript files.
- `scripts/logs/` and `scripts/executions/`: script execution logs and metadata.
- `plugins/`: installed Java plugin jars and old plugin layouts.

Current user data paths must be recognized during migration:

- Windows: `%APPDATA%/CorreoMqtt`
- macOS: `~/Library/Application Support/CorreoMqtt`
- Linux and Unix-like systems: `~/.correomqtt`

## Target Architecture

Use a Cargo workspace with small crates:

```text
crates/
  correo-app/            # binary, eframe bootstrap, dependency assembly
  correo-ui/             # egui screens and view models only
  correo-core/           # app state machine, commands, events, domain models
  correo-mqtt/           # MQTT client trait and implementations
  correo-storage/        # config, secrets, histories, import/export, migration
  correo-scripting/      # JavaScript runtime and host bindings
  correo-plugins/        # plugin manifests, registry, WASM host, built-ins
  correo-diagnostics/    # tracing, log capture, crash reports
xtask/                   # packaging and developer automation
```

Recommended dependency families:

| Area | Crates |
| --- | --- |
| UI | `eframe`, `egui`, `egui_extras`, `egui_dock` or `egui_tiles`, `egui_code_editor`, `rfd`, `arboard`, `image` |
| Async/runtime | `tokio`, `futures`, `tokio-util`, `async-trait`, `flume` or `tokio::sync::mpsc` |
| MQTT | `rumqttc-next` family: `rumqttc-v4-next` for MQTT 3.1.1 and `rumqttc-v5-next` for MQTT 5, hidden behind a local trait |
| Serialization | `serde`, `serde_json`, `serde_with`, `serde_path_to_error`, `schemars`, `uuid`, `time` |
| Storage | `directories`, `tempfile`, `fs4`, `notify`, `zip`, `base64` |
| Secrets | `keyring`, `aes-gcm`, `cbc`, `pbkdf2`, `sha2`, `zeroize`, `rand` |
| TLS/SSH | `rustls`, `rustls-native-certs`, `rustls-pemfile`, `native-tls` where platform stores or PKCS#12 require it, `russh` |
| Scripting | `rquickjs`, `rquickjs-serde` |
| Plugins | `wasmtime`, `wit-bindgen`, `semver` |
| Logging/errors | `tracing`, `tracing-subscriber`, `tracing-appender`, `thiserror`, `color-eyre` |
| Testing | `insta`, `proptest`, `assert_fs`, `wiremock`, MQTT test broker container or local test broker fixture |
| Packaging | `cargo-bundle` or `cargo-packager`, platform-specific signing/notarization tools later |

Crate versions should be pinned in `Cargo.lock` for app builds. Do not hard-code versions in design docs unless the implementation milestone chooses and verifies them.

## Architectural Decisions

### AD-001: egui/eframe desktop shell

Use `eframe` as the native application shell. `correo-ui` owns immediate-mode UI drawing and receives immutable snapshots plus command senders from `correo-core`.

Why:

- The requirement says `egui` must be used.
- The current app has dense tool workflows. egui fits fast, state-driven desktop UI without a browser runtime.
- Keeping UI command-only avoids letting UI widgets own MQTT clients, persistence, or scripting state.

### AD-002: Event-driven core, not Java-style DI

Replace custom Java DI and event bus with explicit Rust ownership:

- `AppModel` stores domain state.
- `AppCommand` messages request work.
- `AppEvent` messages report results.
- Long-running work runs as Tokio tasks owned by service structs.
- UI reads state snapshots and sends commands.

This gives deterministic tests and keeps cross-thread behavior explicit.

### AD-003: MQTT behind a local trait

Define a local `MqttSession` trait in `correo-mqtt` for:

- `connect`
- `disconnect`
- `publish`
- `subscribe`
- `unsubscribe`
- state change events
- incoming message stream

Use the rumqttc-next family initially, but keep it replaceable. The first implementation milestone must prove MQTT 3.1.1 and MQTT 5 coverage for:

- clean session / clean start
- username/password
- QoS 0/1/2 publish and subscribe
- retained messages
- last will
- reconnect reporting
- TLS with host verification toggle
- SSH tunnel routing

### AD-004: Preserve old config, reinitialize plugins

The Rust app must migrate old configuration files. It may drop and reinitialize installed plugins.

Migration policy:

- Never modify old files before creating a timestamped backup directory.
- Load old JSON with permissive serde structs that ignore unknown fields.
- Migrate all known connection fields, settings, themes, UI settings, histories, scripts, and import/export formats.
- Read `passwords.json` and migrate secrets into the new secret store after the user provides or unlocks the old master password.
- Preserve plugin repository settings as metadata when possible.
- Ignore old Java plugin jars and old hook config by default. Create a fresh Rust plugin state from bundled plugins and a new manifest format.
- Record migration warnings in diagnostics and show them in the UI.

### AD-005: Secrets move to OS keyring, file encryption remains import-only

Use the `keyring` crate for new secret storage. Keep file-encryption support only for importing old data and encrypted connection export/import.

Old password compatibility to implement:

- Current format: `AES/GCM/NoPadding`, PBKDF2-HMAC-SHA256, 65536 iterations, 256-bit key, base64 of `iv || salt || ciphertext`.
- Legacy format: `AES/CBC/PKCS5Padding`, PBKDF2-HMAC-SHA512, 40000 iterations, 128-bit key, old `salt:iv:ciphertext` composition.

After successful migration, new secrets should be stored under stable keys derived from connection id and secret type. Keep secret bytes zeroized after use.

### AD-006: WASM plugin ABI, not Rust dynamic libraries

Do not load Rust dynamic libraries as first-class plugins. Rust ABI stability and unsafe process-level access make that too brittle for this app.

Use a Wasmtime-based plugin runtime for portable, sandboxed plugins. Plugin code is built and shipped as separate files, not embedded into the desktop binary. The first plugin ABI should cover non-UI extension points:

- outgoing message transform
- incoming message transform
- message validator
- detail view byte transform
- detail view formatter

Defer full UI plugin injection until the egui shell stabilizes. UI plugin work should use constrained contributions such as toolbar actions, detail panels, and formatter panels rather than arbitrary widget mutation.

Plugin package layout:

```text
plugin.toml
plugin.wasm
assets/
```

`plugin.toml` and `plugin.wasm` are required. `assets/` is optional. Installed plugins are stored under the current profile data directory:

```text
plugins/<plugin-id>/plugin.toml
plugins/<plugin-id>/plugin.wasm
plugins/<plugin-id>/assets/
```

Manifest fields:

- `id`
- `name`
- `version`
- `description`
- `provider`
- `license`
- `compatible_correomqtt`
- `capabilities`
- `entrypoints`
- optional JSON schema for config

Host guarantees:

- Plugins receive JSON-serializable DTOs, not internal Rust structs.
- Plugins cannot access files, network, keyring, or MQTT unless the manifest requests a capability and the host grants it.
- Plugin errors are isolated and become user-visible diagnostics.
- Bundled Java plugins should be reimplemented as WASM plugin packages during migration.

Repository and install model:

- Build/package automation compiles plugin crates to `wasm32-unknown-unknown` and stages package directories next to the executable.
- Build/package automation generates `local-repo.json` next to the executable. Its entries use `local_package` paths relative to that executable directory.
- The binary embeds only `bundled.json`, a list of plugin ids that should be installed automatically. It must not embed plugin manifests or WASM bytes.
- Startup loads repository metadata from `local-repo.json`, configured plugin repositories, and the default repository `https://github.com/EXXETA/correomqtt/releases/download/latest/default-repo.json` when default repositories are enabled.
- Missing, unreachable, or invalid repositories are logged to CLI/tracing and ignored. Repository failures must not abort startup.
- Remote repositories may use archive install sources with `url` and `sha256`; the app downloads, verifies, and extracts them into the profile plugin directory.
- On startup, bundled plugin ids from `bundled.json` are installed automatically when matching repository entries exist and the plugin is not already installed.
- Marketplace install copies local packages or extracts verified archives into the profile plugin directory. Marketplace uninstall removes the installed package directory.

### AD-007: Embedded JavaScript scripting with explicit host API

Use `rquickjs` for embedded JavaScript. Provide a compatibility layer for the current GraalJS script concepts:

- `clientFactory.getBlockingClient()`
- `clientFactory.getAsyncClient()`
- `clientFactory.getPromiseClient()`
- `sleep(ms)`
- `logger`
- `queue.process()`
- `queue.jumpOut()`
- `join()`

Do not expose arbitrary host classes or all-access filesystem/network APIs. Scripts should only talk to CorreoMQTT through explicit host functions:

- connect/disconnect current connection
- publish
- subscribe/unsubscribe
- receive subscription callbacks
- log
- cancellation checks

Script executions should persist:

- execution id
- script file name
- connection id
- start/end time
- status
- error type and message
- log file path

Cancellation must interrupt the JS runtime and cancel outstanding MQTT operations owned by that execution.

### AD-008: Rust-native diagnostics

Use `tracing` for app logs and an in-app diagnostics model for user-visible errors.

Requirements:

- One rotating app log file in the user data directory.
- Per-script log files compatible with the current scripts/logs mental model.
- Redaction for passwords, key material, and MQTT auth values.
- Structured diagnostics for config migration warnings, MQTT connection failures, plugin failures, and script failures.

### AD-009: Import/export compatibility

Keep `.cqc` connection import/export compatible with the current JSON shapes.

Rules:

- Plain export writes `connectionConfigDTOS`.
- Encrypted export writes `encryptionType` and `encryptedData`.
- Encrypted import supports the current AES-GCM format.
- Message import/export preserves `MessageDTO` semantics: topic, payload, retained, QoS, timestamp, message id, message type, publish status.

### AD-010: Build and packaging

Use an `xtask` crate for build automation:

- `cargo xtask check`
- `cargo xtask test`
- `cargo xtask package`
- `cargo xtask migrate-fixtures`

Packaging should first target unsigned dev builds for Windows, macOS, and Linux. Code signing, notarization, and installer auto-update are later release tasks requiring CEO or board direction if they involve external commitments or spend.

## Domain Model

Core DTOs should be Rust structs with serde compatibility:

- `ConnectionConfig`
- `Settings`
- `ThemeSettings`
- `GlobalUiSettings`
- `ConnectionUiSettings`
- `Message`
- `Subscription`
- `HooksConfigLegacy`
- `PasswordFileLegacy`
- `ConnectionExport`
- `ScriptFile`
- `ScriptExecution`
- `PluginManifest`

Keep a separate `legacy` module for old serde shapes. Convert legacy models into current domain models through explicit `TryFrom` implementations so migration failures are testable and user-facing.

## Runtime Flow

Startup:

1. Initialize diagnostics and panic reporting.
2. Resolve user data directory.
3. Acquire a single-instance/storage lock.
4. Detect old Java data files.
5. Run migration if needed, with backup and report.
6. Load current config and secrets metadata.
7. Load plugin repositories from `local-repo.json`, configured repositories, and the default repository when enabled.
8. Auto-install bundled plugin ids from embedded `bundled.json` into the profile plugin directory.
9. Start the plugin registry from installed plugin package directories.
10. Start eframe UI.

Connection flow:

1. UI sends `Connect(connection_id)`.
2. Core validates config and requests secrets from storage.
3. MQTT service opens optional SSH tunnel, builds TLS/auth config, and connects.
4. MQTT service emits state changes and incoming messages.
5. Core applies plugin transforms/validators and updates UI state/history.

Publish flow:

1. UI or script sends `Publish`.
2. Core creates a message id and timestamp.
3. Outgoing plugins transform or reject the message.
4. MQTT service publishes.
5. Core records status and history.

Script flow:

1. UI creates an execution for a selected script and connection.
2. Scripting service starts isolated JS runtime with explicit CorreoMQTT host object.
3. Runtime sends commands through core and receives callbacks through queues.
4. Logs and execution metadata are persisted incrementally.
5. Completion, cancellation, or failure updates UI state.

## File Size and Module Rules

- Maximum Rust source file size: 500 lines.
- Target size: 150-300 lines for most files.
- Split by behavior, not by arbitrary type count.
- Large UI screens should use one file for screen orchestration and separate files for panels, rows, dialogs, and state adapters.
- Large serde models should split current and legacy schemas.
- Tests may live in `tests/` or focused `mod tests` blocks, but test files should also stay under 500 lines.

## Milestones

### M0: Strategy and working agreements

Deliverables:

- `architecture.md`
- `AGENTS.md`

Verification:

- Docs exist at repo root and cover tech stack, crate choices, decisions, plugins, scripting, migration, milestones, and file-size rule.

### M1: Rust workspace and migration fixtures

Deliverables:

- Cargo workspace skeleton.
- Domain models and legacy serde models.
- Fixture copies of old `config.json`, `passwords.json`, `hooks.json`, histories, scripts, and `.cqc` exports.
- Migration report type.

Acceptance:

- `cargo test -p correo-storage` migrates representative old configs without data loss for known fields.
- Old Java plugin files are ignored and a fresh plugin state is produced.
- Password import tests cover AES-GCM and legacy AES-CBC.

### M2: MQTT engine

Deliverables:

- `MqttSession` trait.
- MQTT 3.1.1 and MQTT 5 implementation.
- Connection lifecycle and state events.
- TLS and SSH tunnel proof of concept.

Acceptance:

- Local broker tests cover connect, disconnect, publish, subscribe, retained, QoS, and reconnect state.
- Errors are typed and redacted.

### M3: egui application shell

Deliverables:

- eframe app bootstrap.
- Main connection list.
- Connection settings editor.
- Basic logs/diagnostics panel.

Acceptance:

- User can create, edit, delete, and persist connections.
- Existing migrated config opens in the UI.

### M4: Publish/subscribe workflows

Deliverables:

- Connection detail workspace.
- Subscribe panel.
- Publish panel.
- Message list and detail view.
- Per-connection histories.

Acceptance:

- User can connect to a broker, subscribe, receive messages, publish messages, and see histories persist across restart.

### M5: Scripting MVP

Deliverables:

- Script file browser/editor.
- rquickjs runtime.
- Compatibility host API.
- Script execution logs and cancellation.

Acceptance:

- Existing simple scripts using connect, publish, subscribe, queue, and promise-style calls can be adapted or run through compatibility aliases.
- Script failures show host vs guest error category.

### M6: Plugin MVP

Deliverables:

- WASM plugin manifest and loader.
- Separate WASM plugin package build outputs for bundled formatters/manipulators/validators.
- `local-repo.json` generation next to the executable during packaging.
- Embedded `bundled.json` listing plugin ids to auto-install.
- Marketplace repository loading from `local-repo.json`, configured repositories, and the default repository.
- Plugin config UI sufficient for message transforms and validators.

Acceptance:

- Incoming/outgoing transforms, validators, and detail formatters work without Java plugins.
- Bundled plugins auto-install from package files on startup without embedding WASM bytes in the binary.
- Invalid or unavailable repositories are logged and ignored without failing startup.
- Old plugin folders can be deleted or left unused without breaking startup.

### M7: Import/export and settings parity

Deliverables:

- `.cqc` import/export.
- Message import/export.
- Settings, language, theme, search options, keyring selection.

Acceptance:

- Current Java exports import into Rust.
- Rust exports import into Rust and, where schema-compatible, back into current Java.

### M8: Beta hardening and packaging

Deliverables:

- Platform packages.
- Migration rollback path.
- QA regression suite.
- Release notes listing dropped Java plugin compatibility.

Acceptance:

- Fresh install and migrated install pass QA on Windows, macOS, and Linux.
- No known secret leakage in logs or diagnostics.

## Key Risks

- MQTT 5 parity: verify rumqttc-next behavior early. If gaps appear, keep the `MqttSession` trait and swap implementation without touching UI.
- TLS keystore compatibility: Java keystores may not map cleanly to Rust TLS. Migration should preserve paths/passwords and report unsupported formats rather than silently dropping them.
- Scripting compatibility: GraalJS allowed full host access. Rust must intentionally narrow this for security, which may break advanced scripts.
- UI plugin parity: JavaFX plugin injection cannot be carried over directly. Treat UI plugins as a new product surface, not a migration guarantee.
- Rewrite scope: keep milestones vertical. Do not build all UI before proving MQTT, storage, scripting, and plugin seams.

## Verification Strategy

- Unit tests for domain conversion and config migration.
- Golden snapshot tests for legacy JSON inputs and migrated JSON outputs.
- Integration tests against a local MQTT broker.
- Script runtime tests for blocking, async, promise, queue, cancellation, and logging.
- Plugin ABI tests with small WASM fixture plugins.
- Manual QA scripts for main user workflows before beta packaging.
