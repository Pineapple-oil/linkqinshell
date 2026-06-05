use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, Result};

pub type ConnectionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    Password,
    PrivateKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshAuthConfig {
    pub method: AuthMethod,
    pub username: String,
    pub encrypted_password: Option<String>,
    pub private_key_path: Option<String>,
    pub encrypted_private_key: Option<String>,
    pub encrypted_private_key_passphrase: Option<String>,
    pub remember_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProxyConfig {
    None,
    Http {
        host: String,
        port: u16,
        username: Option<String>,
        encrypted_password: Option<String>,
    },
    Socks5 {
        host: String,
        port: u16,
        username: Option<String>,
        encrypted_password: Option<String>,
    },
    SshJump {
        connection_id: ConnectionId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoftDeleteState {
    pub deleted_at: Option<DateTime<Utc>>,
    pub recover_until: Option<DateTime<Utc>>,
}

impl SoftDeleteState {
    pub fn active() -> Self {
        Self {
            deleted_at: None,
            recover_until: None,
        }
    }

    pub fn deleted(now: DateTime<Utc>) -> Self {
        Self {
            deleted_at: Some(now),
            recover_until: Some(now + Duration::days(30)),
        }
    }

    pub fn is_recoverable_at(&self, now: DateTime<Utc>) -> bool {
        matches!(self.recover_until, Some(until) if now <= until)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Connection {
    pub id: ConnectionId,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub auth: SshAuthConfig,
    pub proxy: ProxyConfig,
    pub group_id: Option<Uuid>,
    pub tag_ids: Vec<Uuid>,
    pub favorite: bool,
    pub recent_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub terminal_type: String,
    pub encoding: String,
    pub startup_directory: Option<String>,
    pub post_login_command: Option<String>,
    pub default_download_dir: Option<String>,
    pub soft_delete: SoftDeleteState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Connection {
    pub fn new_ssh(name: impl Into<String>, host: impl Into<String>, auth: SshAuthConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            host: host.into(),
            port: 22,
            auth,
            proxy: ProxyConfig::None,
            group_id: None,
            tag_ids: Vec::new(),
            favorite: false,
            recent_at: None,
            notes: None,
            terminal_type: "xterm-256color".to_string(),
            encoding: "utf-8".to_string(),
            startup_directory: None,
            post_login_command: None,
            default_download_dir: None,
            soft_delete: SoftDeleteState::active(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("connection name is required".into()));
        }
        if self.host.trim().is_empty() {
            return Err(AppError::Validation("host is required".into()));
        }
        if self.port == 0 {
            return Err(AppError::Validation(
                "port must be between 1 and 65535".into(),
            ));
        }
        if self.auth.username.trim().is_empty() {
            return Err(AppError::Validation("username is required".into()));
        }
        match self.auth.method {
            AuthMethod::Password if self.auth.encrypted_password.is_none() => {
                return Err(AppError::Validation(
                    "password credential is required".into(),
                ));
            }
            AuthMethod::PrivateKey
                if self.auth.private_key_path.is_none()
                    && self.auth.encrypted_private_key.is_none() =>
            {
                return Err(AppError::Validation(
                    "private key path or content is required".into(),
                ));
            }
            _ => {}
        }
        Ok(())
    }

    pub fn mark_deleted(&mut self, now: DateTime<Utc>) {
        self.soft_delete = SoftDeleteState::deleted(now);
        self.updated_at = now;
    }

    pub fn restore(&mut self, now: DateTime<Utc>) -> Result<()> {
        if !self.soft_delete.is_recoverable_at(now) {
            return Err(AppError::Validation(
                "connection is no longer recoverable".into(),
            ));
        }
        self.soft_delete = SoftDeleteState::active();
        self.updated_at = now;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionGroup {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionTag {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password_auth() -> SshAuthConfig {
        SshAuthConfig {
            method: AuthMethod::Password,
            username: "root".into(),
            encrypted_password: Some("ciphertext".into()),
            private_key_path: None,
            encrypted_private_key: None,
            encrypted_private_key_passphrase: None,
            remember_secret: true,
        }
    }

    #[test]
    fn validates_required_ssh_fields() {
        let connection = Connection::new_ssh("", "example.com", password_auth());
        let error = connection.validate().unwrap_err().to_string();
        assert!(error.contains("connection name is required"));
    }

    #[test]
    fn password_auth_requires_encrypted_password() {
        let mut auth = password_auth();
        auth.encrypted_password = None;
        let connection = Connection::new_ssh("prod", "example.com", auth);
        let error = connection.validate().unwrap_err().to_string();
        assert!(error.contains("password credential is required"));
    }

    #[test]
    fn soft_deleted_connection_is_recoverable_for_thirty_days() {
        let now = Utc::now();
        let mut connection = Connection::new_ssh("prod", "example.com", password_auth());
        connection.mark_deleted(now);
        assert!(connection
            .soft_delete
            .is_recoverable_at(now + Duration::days(30)));
        assert!(!connection
            .soft_delete
            .is_recoverable_at(now + Duration::days(31)));
    }
}
