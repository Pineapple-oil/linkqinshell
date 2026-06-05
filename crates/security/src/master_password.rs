use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use linkqinshell_core::{AppError, Result, SecretStore};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedPayload {
    pub version: u8,
    pub kdf: String,
    pub cipher: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Clone)]
pub struct MasterPasswordSecretStore {
    master_password: String,
}

impl MasterPasswordSecretStore {
    pub fn new(master_password: impl Into<String>) -> Result<Self> {
        let master_password = master_password.into();
        if master_password.trim().len() < 8 {
            return Err(AppError::Security(
                "master password must be at least 8 characters".into(),
            ));
        }
        Ok(Self { master_password })
    }

    fn derive_key(&self, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
        let mut key = [0_u8; KEY_LEN];
        Argon2::default()
            .hash_password_into(self.master_password.as_bytes(), salt, &mut key)
            .map_err(|error| AppError::Security(format!("key derivation failed: {error}")))?;
        Ok(key)
    }

    fn encode(payload: &EncryptedPayload) -> Result<String> {
        serde_json::to_string(payload)
            .map_err(|error| AppError::Security(format!("payload encode failed: {error}")))
    }

    fn decode(payload: &str) -> Result<EncryptedPayload> {
        serde_json::from_str(payload)
            .map_err(|error| AppError::Security(format!("payload decode failed: {error}")))
    }
}

#[async_trait]
impl SecretStore for MasterPasswordSecretStore {
    async fn encrypt(&self, plain: &[u8]) -> Result<String> {
        let mut salt = [0_u8; SALT_LEN];
        let mut nonce = [0_u8; NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let mut key = self.derive_key(&salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| AppError::Security(format!("cipher init failed: {error}")))?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plain)
            .map_err(|error| AppError::Security(format!("encrypt failed: {error}")))?;
        key.zeroize();

        Self::encode(&EncryptedPayload {
            version: VERSION,
            kdf: "argon2id-default".into(),
            cipher: "aes-256-gcm".into(),
            salt: STANDARD.encode(salt),
            nonce: STANDARD.encode(nonce),
            ciphertext: STANDARD.encode(ciphertext),
        })
    }

    async fn decrypt(&self, secret: &str) -> Result<Vec<u8>> {
        let payload = Self::decode(secret)?;
        if payload.version != VERSION {
            return Err(AppError::Security(
                "unsupported encrypted payload version".into(),
            ));
        }

        let salt = STANDARD
            .decode(payload.salt)
            .map_err(|error| AppError::Security(format!("salt decode failed: {error}")))?;
        let nonce = STANDARD
            .decode(payload.nonce)
            .map_err(|error| AppError::Security(format!("nonce decode failed: {error}")))?;
        let ciphertext = STANDARD
            .decode(payload.ciphertext)
            .map_err(|error| AppError::Security(format!("ciphertext decode failed: {error}")))?;

        let mut key = self.derive_key(&salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| AppError::Security(format!("cipher init failed: {error}")))?;
        let plain = cipher
            .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| {
                AppError::Security("decrypt failed; master password may be wrong".into())
            })?;
        key.zeroize();
        Ok(plain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn decrypts_secret_with_same_master_password() {
        let store = MasterPasswordSecretStore::new("correct horse battery staple").unwrap();
        let ciphertext = store.encrypt(b"server-password").await.unwrap();
        let plain = store.decrypt(&ciphertext).await.unwrap();
        assert_eq!(plain, b"server-password");
        assert!(!ciphertext.contains("server-password"));
    }

    #[tokio::test]
    async fn rejects_wrong_master_password() {
        let store = MasterPasswordSecretStore::new("correct horse battery staple").unwrap();
        let ciphertext = store.encrypt(b"server-password").await.unwrap();
        let wrong = MasterPasswordSecretStore::new("incorrect horse battery staple").unwrap();
        let error = wrong.decrypt(&ciphertext).await.unwrap_err().to_string();
        assert!(error.contains("decrypt failed"));
    }
}
