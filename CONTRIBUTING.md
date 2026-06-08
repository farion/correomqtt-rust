# Contributing to CorreoMQTT

CorreoMQTT is currently a Rust port. Use the workspace and architecture notes
in this repository as the source of truth before opening a change.

## Before Opening a Change

- Check existing issues in `https://github.com/farion/correomqtt-rust/issues`.
- Keep Rust source files under 500 lines.
- Keep UI code in `correo-ui`; do not mix service logic into egui widgets.
- Use synthetic credentials in tests and fixtures.

## Verification

Run the smallest command that proves your change. Useful defaults:

```bash
cargo fmt
cargo test --locked -p xtask
```

For packaging changes, include the target triple, command, output artifact, and
known platform caveats in the pull request.
