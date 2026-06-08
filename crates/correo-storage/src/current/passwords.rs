use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordFile {
    pub encryption: PasswordEncryption,
    pub encrypted_payload: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordEncryption {
    AesGcmNoPadding,
    AesCbcPkcs5Padding,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretMaterial(String);

impl SecretMaterial {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_for_migration(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretMaterial(<redacted>)")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedSecret {
    pub reference: SecretReference,
    pub value: SecretMaterial,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretReference {
    pub connection_id: String,
    pub kind: SecretKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecretKind {
    Password,
    AuthPassword,
    SslKeystorePassword,
}
