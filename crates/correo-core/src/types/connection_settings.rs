use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionSettingsTab {
    #[default]
    Mqtt,
    Tls,
    Proxy,
    Lwt,
}

impl ConnectionSettingsTab {
    pub const ALL: [Self; 4] = [Self::Mqtt, Self::Tls, Self::Proxy, Self::Lwt];

    pub fn label(self) -> &'static str {
        match self {
            Self::Mqtt => "MQTT",
            Self::Tls => "TLS/SSL",
            Self::Proxy => "Proxy/Tunnel",
            Self::Lwt => "LWT",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyringState {
    #[default]
    Available,
    Locked,
    Unavailable,
    MigrationRequired,
}

impl KeyringState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Keyring available",
            Self::Locked => "Keyring locked",
            Self::Unavailable => "Keyring unavailable",
            Self::MigrationRequired => "Keyring migration required",
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct SecretInput {
    value: String,
}

impl SecretInput {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn expose_for_ui(&self) -> &str {
        &self.value
    }

    pub fn expose_non_empty(&self) -> Option<&str> {
        let value = self.value.trim();
        (!value.is_empty()).then_some(value)
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl fmt::Debug for SecretInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.is_empty() {
            formatter.write_str("SecretInput(<empty>)")
        } else {
            formatter.write_str("SecretInput(<redacted>)")
        }
    }
}

impl Serialize for SecretInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(if self.value.is_empty() {
            ""
        } else {
            "<redacted>"
        })
    }
}

impl<'de> Deserialize<'de> for SecretInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = Option::<String>::deserialize(deserializer)?;
        Ok(Self::default())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionSettingsSnapshot {
    pub selected_tab: ConnectionSettingsTab,
    pub internal_id: String,
    pub profile_name: String,
    pub host: String,
    pub port: String,
    pub mqtt_version: String,
    pub clean_session: bool,
    pub client_id: String,
    pub username: String,
    pub password: SecretInput,
    pub password_status: String,
    pub tls_mode: String,
    pub tls_store: String,
    pub tls_keystore_password: SecretInput,
    pub tls_password_status: String,
    pub tls_host_verification: bool,
    pub proxy_mode: String,
    pub ssh_host: String,
    pub ssh_port: String,
    pub local_mqtt_port: String,
    pub auth_mode: String,
    pub auth_username: String,
    pub ssh_password: SecretInput,
    pub ssh_password_status: String,
    pub ssh_key_file: String,
    pub lwt_enabled: bool,
    pub lwt_topic: String,
    pub lwt_retained: bool,
    pub lwt_payload: String,
    pub dirty: bool,
    pub valid: bool,
    pub save_disabled_reason: String,
    pub delete_confirmation_open: bool,
    pub keyring_state: KeyringState,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionSettingField {
    ProfileName,
    Host,
    Port,
    MqttVersion,
    ClientId,
    Username,
    TlsMode,
    TlsStore,
    ProxyMode,
    SshHost,
    SshPort,
    LocalMqttPort,
    AuthMode,
    AuthUsername,
    SshKeyFile,
    LwtTopic,
    LwtPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSettingFlag {
    CleanSession,
    TlsHostVerification,
    LwtRetained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSecretField {
    MqttPassword,
    TlsKeystorePassword,
    SshPassword,
}
