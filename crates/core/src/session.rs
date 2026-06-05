use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type SessionId = Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Connecting,
    HostKeyVerification,
    Authenticating,
    Connected,
    Reconnecting,
    Disconnecting,
    Closed,
    Failed,
}

impl SessionState {
    pub fn can_reconnect(self) -> bool {
        matches!(self, Self::Connected | Self::Failed | Self::Closed)
    }
}
