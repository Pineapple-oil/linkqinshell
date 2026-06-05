use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    ConnectionCreated,
    ConnectionUpdated,
    ConnectionDeleted,
    ConnectionRestored,
    HostKeyTrusted,
    HostKeyChangedBlocked,
    SftpUpload,
    SftpDownload,
    DangerousCommandConfirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: Uuid,
    pub action: AuditAction,
    pub connection_id: Option<Uuid>,
    pub message: String,
    pub created_at: DateTime<Utc>,
}
