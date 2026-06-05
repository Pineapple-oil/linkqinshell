use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostKey {
    pub id: Uuid,
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub fingerprint_sha256: String,
    pub fingerprint_md5: Option<String>,
    pub public_key: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub trusted: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostKeyStatus {
    Unknown,
    Trusted,
    Changed,
    Revoked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostKeyDecision {
    TrustNew,
    Reject,
    UpdateTrustedKey,
}

impl HostKey {
    pub fn compare_to(&self, observed_fingerprint: &str) -> HostKeyStatus {
        if !self.trusted {
            return HostKeyStatus::Revoked;
        }
        if self.fingerprint_sha256 == observed_fingerprint {
            HostKeyStatus::Trusted
        } else {
            HostKeyStatus::Changed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_host_key_is_blocked_status() {
        let host_key = HostKey {
            id: Uuid::new_v4(),
            host: "example.com".into(),
            port: 22,
            key_type: "ssh-ed25519".into(),
            fingerprint_sha256: "SHA256:old".into(),
            fingerprint_md5: None,
            public_key: "public".into(),
            first_seen_at: Utc::now(),
            last_seen_at: Utc::now(),
            trusted: true,
        };

        assert_eq!(host_key.compare_to("SHA256:new"), HostKeyStatus::Changed);
    }
}
