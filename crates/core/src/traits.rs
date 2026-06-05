use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    AuditEvent, Connection, ConnectionId, HostKey, HostKeyDecision, Result, SessionState,
    TransferConflictPolicy, TransferTask,
};

#[derive(Debug, Clone)]
pub struct TerminalChunk {
    pub session_id: Uuid,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SshSessionConfig {
    pub connection: Connection,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub id: Uuid,
    pub state: SessionState,
}

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: Option<u32>,
    pub modified_at: Option<i64>,
}

#[async_trait]
pub trait ConnectionRepository: Send + Sync {
    async fn save(&self, connection: &Connection) -> Result<()>;
    async fn find_by_id(&self, id: &ConnectionId) -> Result<Option<Connection>>;
    async fn list_active(&self) -> Result<Vec<Connection>>;
    async fn soft_delete(&self, id: &ConnectionId) -> Result<()>;
    async fn restore(&self, id: &ConnectionId) -> Result<()>;
}

#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn encrypt(&self, plain: &[u8]) -> Result<String>;
    async fn decrypt(&self, secret: &str) -> Result<Vec<u8>>;
}

#[async_trait]
pub trait HostKeyRepository: Send + Sync {
    async fn find(&self, host: &str, port: u16) -> Result<Option<HostKey>>;
    async fn trust(&self, key: &HostKey, decision: HostKeyDecision) -> Result<()>;
}

#[async_trait]
pub trait SshBackend: Send + Sync {
    async fn connect(&self, config: SshSessionConfig) -> Result<SessionHandle>;
    async fn verify_host_key(&self, host: &str, port: u16) -> Result<HostKey>;
    async fn write_stdin(&self, session_id: Uuid, bytes: &[u8]) -> Result<()>;
    async fn disconnect(&self, session_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait SftpBackend: Send + Sync {
    async fn list_dir(&self, config: SshSessionConfig, path: &str) -> Result<Vec<RemoteFile>>;
    async fn upload(
        &self,
        config: SshSessionConfig,
        local_path: &str,
        remote_path: &str,
        conflict_policy: TransferConflictPolicy,
    ) -> Result<TransferTask>;
    async fn download(
        &self,
        config: SshSessionConfig,
        remote_path: &str,
        local_path: &str,
        conflict_policy: TransferConflictPolicy,
    ) -> Result<TransferTask>;
    async fn rename(&self, config: SshSessionConfig, from: &str, to: &str) -> Result<()>;
    async fn mkdir(&self, config: SshSessionConfig, path: &str) -> Result<()>;
    async fn delete(&self, config: SshSessionConfig, path: &str) -> Result<()>;
}

#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn record(&self, event: AuditEvent) -> Result<()>;
}
