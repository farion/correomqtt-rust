use std::fmt;

const REDACTED: &str = "<redacted>";

#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString(<redacted>)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretBytes(<redacted>)")
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for SecretBytes {
    fn from(value: &[u8]) -> Self {
        Self::new(value)
    }
}
