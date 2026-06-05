use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("authentication failed: {0}")]
    Authentication(String),
    #[error("host key verification failed: {0}")]
    HostKey(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("security error: {0}")]
    Security(String),
    #[error("ssh error: {0}")]
    Ssh(String),
    #[error("sftp error: {0}")]
    Sftp(String),
    #[error("terminal error: {0}")]
    Terminal(String),
    #[error("io error: {0}")]
    Io(String),
}
