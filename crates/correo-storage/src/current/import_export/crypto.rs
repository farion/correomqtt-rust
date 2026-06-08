use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::{Result, StorageError};

const GCM_IV_BYTES: usize = 12;
const GCM_SALT_BYTES: usize = 16;

pub(super) fn encrypt_gcm(plaintext: &str, password: &str) -> Result<String> {
    let mut salt = [0u8; GCM_SALT_BYTES];
    let mut iv = [0u8; GCM_IV_BYTES];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 65_536, &mut key);
    let cipher = cipher_from_key(&mut key)?;
    let encrypted = cipher.encrypt(Nonce::from_slice(&iv), plaintext.as_bytes());
    key.zeroize();
    let ciphertext = encrypted.map_err(|_| StorageError::ConnectionExportDecryption)?;

    let mut payload = Vec::with_capacity(GCM_IV_BYTES + GCM_SALT_BYTES + ciphertext.len());
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(payload))
}

pub(super) fn decrypt_gcm(encrypted_data: &str, password: &str) -> Result<String> {
    let payload = STANDARD.decode(encrypted_data).map_err(|_| {
        StorageError::InvalidConnectionExportPayload("AES-GCM payload is not base64")
    })?;
    if payload.len() <= GCM_IV_BYTES + GCM_SALT_BYTES {
        return Err(StorageError::InvalidConnectionExportPayload(
            "AES-GCM payload is too short",
        ));
    }

    let (iv, rest) = payload.split_at(GCM_IV_BYTES);
    let (salt, ciphertext) = rest.split_at(GCM_SALT_BYTES);
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 65_536, &mut key);
    let cipher = cipher_from_key(&mut key)?;
    let decrypted = cipher.decrypt(Nonce::from_slice(iv), ciphertext);
    key.zeroize();
    let plaintext = decrypted.map_err(|_| StorageError::ConnectionExportDecryption)?;
    String::from_utf8(plaintext).map_err(|_| StorageError::ConnectionExportDecryption)
}

fn cipher_from_key(key: &mut [u8; 32]) -> Result<Aes256Gcm> {
    match Aes256Gcm::new_from_slice(key) {
        Ok(cipher) => Ok(cipher),
        Err(_) => {
            key.zeroize();
            Err(StorageError::ConnectionExportDecryption)
        }
    }
}
