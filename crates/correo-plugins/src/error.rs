use crate::{
    HookKind, IntoPluginDiagnostic, PackageError, PluginDiagnostic, PluginDiagnosticSeverity,
};
use semver::Version;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeLoadError {
    #[error(transparent)]
    Package(#[from] PackageError),
    #[error("failed to create Wasmtime engine: {0}")]
    Engine(wasmtime::Error),
    #[error("plugin {plugin_id} is not compatible with CorreoMQTT {correo_version}")]
    IncompatibleCorreoVersion {
        plugin_id: String,
        correo_version: Version,
    },
    #[error("plugin {plugin_id} requests unsupported host capability {surface:?}")]
    UnsupportedHostCapability {
        plugin_id: String,
        surface: crate::HostSurface,
    },
    #[error("failed to compile plugin WASM: {0}")]
    Compile(wasmtime::Error),
    #[error("plugin WASM imports denied host surface {module}.{name}")]
    HostImportDenied { module: String, name: String },
    #[error("plugin WASM must export linear memory named `memory`")]
    MissingMemoryExport,
    #[error("plugin WASM memory must be unshared 32-bit memory")]
    UnsupportedMemoryExport,
    #[error("plugin WASM initial memory {initial_pages} pages exceeds limit {max_pages} pages")]
    InitialMemoryTooLarge { initial_pages: u64, max_pages: u64 },
    #[error("plugin WASM must export allocator `correomqtt_alloc`")]
    MissingAllocatorExport,
    #[error("plugin WASM export `{export}` has invalid signature")]
    InvalidExportSignature { export: String },
    #[error("plugin WASM is missing entrypoint export `{export}` for {hook:?}")]
    MissingEntrypointExport { hook: HookKind, export: String },
}

impl IntoPluginDiagnostic for RuntimeLoadError {
    fn diagnostic(&self) -> PluginDiagnostic {
        PluginDiagnostic::error(self.to_string())
    }
}

#[derive(Debug, Error)]
pub enum HookDispatchError {
    #[error("plugin hook {hook:?} is not declared in the manifest")]
    HookNotDeclared { hook: HookKind },
    #[error("plugin hook {hook:?} was cancelled")]
    Cancelled { hook: HookKind },
    #[error("plugin hook {hook:?} exceeded the fuel budget")]
    FuelExhausted { hook: HookKind },
    #[error("failed to serialize request for plugin hook {hook:?}: {source}")]
    SerializeRequest {
        hook: HookKind,
        source: serde_json::Error,
    },
    #[error("plugin hook {hook:?} payload is {actual} bytes, over the {limit} byte limit")]
    PayloadTooLarge {
        hook: HookKind,
        actual: usize,
        limit: usize,
    },
    #[error("failed to configure Wasmtime store for hook {hook:?}: {source}")]
    ConfigureStore {
        hook: HookKind,
        source: wasmtime::Error,
    },
    #[error("failed to instantiate plugin WASM: {0}")]
    Instantiate(wasmtime::Error),
    #[error("plugin WASM instance is missing exported memory")]
    MissingMemory,
    #[error("failed to resolve plugin entrypoint: {0}")]
    Entrypoint(wasmtime::Error),
    #[error("failed to allocate guest memory for hook {hook:?}: {source}")]
    Allocate {
        hook: HookKind,
        source: wasmtime::Error,
    },
    #[error("guest allocator returned invalid pointer {ptr} for hook {hook:?}")]
    InvalidGuestPointer { hook: HookKind, ptr: i32 },
    #[error("failed to write request bytes for hook {hook:?}: {source}")]
    MemoryWrite {
        hook: HookKind,
        source: wasmtime::MemoryAccessError,
    },
    #[error("plugin hook {hook:?} trapped: {source}")]
    GuestCall {
        hook: HookKind,
        source: wasmtime::Error,
    },
    #[error("failed to deallocate guest memory for hook {hook:?}: {source}")]
    Deallocate {
        hook: HookKind,
        source: wasmtime::Error,
    },
    #[error("plugin hook {hook:?} returned invalid response pointer {ptr}")]
    InvalidResponsePointer { hook: HookKind, ptr: u32 },
    #[error("failed to read response bytes for hook {hook:?}: {source}")]
    MemoryRead {
        hook: HookKind,
        source: wasmtime::MemoryAccessError,
    },
    #[error("response for plugin hook {hook:?} was not UTF-8 JSON: {source}")]
    ResponseUtf8 {
        hook: HookKind,
        source: FromUtf8Error,
    },
    #[error("failed to decode response for plugin hook {hook:?}: {source}")]
    DecodeResponse {
        hook: HookKind,
        source: serde_json::Error,
    },
    #[error("plugin hook {hook:?} returned ABI version {found}; expected {expected}")]
    AbiVersionMismatch {
        hook: HookKind,
        found: u16,
        expected: u16,
    },
}

impl IntoPluginDiagnostic for HookDispatchError {
    fn diagnostic(&self) -> PluginDiagnostic {
        let diagnostic = PluginDiagnostic::error(self.to_string());
        match self.hook() {
            Some(hook) => diagnostic.for_hook(hook),
            None => diagnostic,
        }
    }
}

impl HookDispatchError {
    pub fn hook(&self) -> Option<HookKind> {
        match self {
            Self::HookNotDeclared { hook }
            | Self::Cancelled { hook }
            | Self::FuelExhausted { hook }
            | Self::SerializeRequest { hook, .. }
            | Self::PayloadTooLarge { hook, .. }
            | Self::ConfigureStore { hook, .. }
            | Self::Allocate { hook, .. }
            | Self::InvalidGuestPointer { hook, .. }
            | Self::MemoryWrite { hook, .. }
            | Self::GuestCall { hook, .. }
            | Self::Deallocate { hook, .. }
            | Self::InvalidResponsePointer { hook, .. }
            | Self::MemoryRead { hook, .. }
            | Self::ResponseUtf8 { hook, .. }
            | Self::DecodeResponse { hook, .. }
            | Self::AbiVersionMismatch { hook, .. } => Some(*hook),
            Self::Instantiate(_) | Self::MissingMemory | Self::Entrypoint(_) => None,
        }
    }

    pub fn severity(&self) -> PluginDiagnosticSeverity {
        PluginDiagnosticSeverity::Error
    }
}
