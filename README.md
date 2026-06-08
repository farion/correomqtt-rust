# CorreoMQTT Rust Port

CorreoMQTT is being ported from the legacy Java/JavaFX application to a native
Rust desktop application built with `egui` through `eframe`.

The Rust workspace is split into focused crates:

- `correo-app`: desktop bootstrap and service assembly
- `correo-ui`: egui screens, panels, dialogs, and view models
- `correo-core`: commands, events, app state, and domain orchestration
- `correo-mqtt`: MQTT traits and client implementations
- `correo-storage`: config, secrets, histories, imports, exports, and migration
- `correo-scripting`: JavaScript runtime and host bindings
- `correo-plugins`: WASM plugin runtime, manifests, and built-ins
- `correo-diagnostics`: tracing, log capture, and diagnostics
- `xtask`: packaging and developer automation

See [architecture.md](architecture.md) for the port direction and compatibility
constraints.

## Development

Use the committed stable toolchain:

```bash
cargo fmt
cargo test --locked -p xtask
```

Run the desktop shell locally with:

```bash
cargo run -p correo-app
```

## Unsigned Beta Packaging

Unsigned internal beta packages are built through `xtask`:

```bash
cargo xtask package
```

See [docs/beta-packaging.md](docs/beta-packaging.md) for target triples,
artifact names, checksums, package layouts, and CI assumptions. See
[docs/beta-release-notes.md](docs/beta-release-notes.md) for beta
compatibility caveats, migration and rollback behavior, diagnostics, and known
limitations.

## Legacy Data

The Rust app detects legacy CorreoMQTT data roots during migration:

- Windows: `%APPDATA%/CorreoMqtt`
- macOS: `~/Library/Application Support/CorreoMqtt`
- Linux/Unix: `~/.correomqtt`

Old Java plugin jars and PF4J metadata are not compatibility targets. Rust
plugin state is reinitialized from Rust plugin manifests and bundled
replacements.

## License

Licensed under GPL v3.0 or later.
