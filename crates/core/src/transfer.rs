use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferState {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Canceling,
    Canceled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferConflictPolicy {
    Overwrite,
    Skip,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferDecision {
    WriteTo(String),
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferTask {
    pub id: Uuid,
    pub direction: TransferDirection,
    pub source: String,
    pub destination: String,
    pub state: TransferState,
    pub progress: f32,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl TransferConflictPolicy {
    pub fn decide(self, destination: &str, destination_exists: bool) -> TransferDecision {
        if !destination_exists {
            return TransferDecision::WriteTo(destination.to_string());
        }
        match self {
            Self::Overwrite => TransferDecision::WriteTo(destination.to_string()),
            Self::Skip => TransferDecision::Skip,
            Self::Rename => {
                let renamed = format!("{destination}.copy");
                TransferDecision::WriteTo(renamed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_policy_skips_existing_destination() {
        assert_eq!(
            TransferConflictPolicy::Skip.decide("/tmp/file.txt", true),
            TransferDecision::Skip
        );
    }

    #[test]
    fn rename_policy_writes_copy_for_existing_destination() {
        assert_eq!(
            TransferConflictPolicy::Rename.decide("/tmp/file.txt", true),
            TransferDecision::WriteTo("/tmp/file.txt.copy".into())
        );
    }
}
