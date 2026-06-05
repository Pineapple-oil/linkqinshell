use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use async_trait::async_trait;
use linkqinshell_core::{
    AppError, RemoteFile, Result, SftpBackend, SshSessionConfig, TransferConflictPolicy,
    TransferDirection, TransferState, TransferTask,
};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Ssh2SftpBackend;

#[async_trait]
impl SftpBackend for Ssh2SftpBackend {
    async fn list_dir(&self, config: SshSessionConfig, path: &str) -> Result<Vec<RemoteFile>> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            let sftp = session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?;
            let entries = sftp
                .readdir(Path::new(&path))
                .map_err(|error| AppError::Sftp(format!("read dir failed: {error}")))?;
            Ok(entries
                .into_iter()
                .map(|(path, stat)| RemoteFile {
                    name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string()),
                    path: path.to_string_lossy().to_string(),
                    is_dir: is_dir(stat.perm),
                    size: stat.size.unwrap_or_default(),
                    permissions: stat.perm,
                    modified_at: stat.mtime.map(|mtime| mtime as i64),
                })
                .collect())
        })
        .await
        .map_err(|error| AppError::Sftp(format!("sftp task failed: {error}")))?
    }

    async fn upload(
        &self,
        config: SshSessionConfig,
        local_path: &str,
        remote_path: &str,
        conflict_policy: TransferConflictPolicy,
    ) -> Result<TransferTask> {
        let local_path = local_path.to_string();
        let remote_path = remote_path.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            let sftp = session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?;
            let exists = sftp.stat(Path::new(&remote_path)).is_ok();
            let destination = match conflict_policy.decide(&remote_path, exists) {
                linkqinshell_core::TransferDecision::WriteTo(path) => path,
                linkqinshell_core::TransferDecision::Skip => {
                    return Ok(task(
                        TransferDirection::Upload,
                        &local_path,
                        &remote_path,
                        TransferState::Completed,
                    ))
                }
            };
            let mut source = File::open(&local_path)
                .map_err(|error| AppError::Io(format!("open local file failed: {error}")))?;
            let mut bytes = Vec::new();
            source
                .read_to_end(&mut bytes)
                .map_err(|error| AppError::Io(format!("read local file failed: {error}")))?;
            let mut remote = sftp
                .create(Path::new(&destination))
                .map_err(|error| AppError::Sftp(format!("create remote file failed: {error}")))?;
            remote
                .write_all(&bytes)
                .map_err(|error| AppError::Sftp(format!("write remote file failed: {error}")))?;
            Ok(task(
                TransferDirection::Upload,
                &local_path,
                &destination,
                TransferState::Completed,
            ))
        })
        .await
        .map_err(|error| AppError::Sftp(format!("upload task failed: {error}")))?
    }

    async fn download(
        &self,
        config: SshSessionConfig,
        remote_path: &str,
        local_path: &str,
        conflict_policy: TransferConflictPolicy,
    ) -> Result<TransferTask> {
        let remote_path = remote_path.to_string();
        let local_path = local_path.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            let sftp = session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?;
            let exists = Path::new(&local_path).exists();
            let destination = match conflict_policy.decide(&local_path, exists) {
                linkqinshell_core::TransferDecision::WriteTo(path) => path,
                linkqinshell_core::TransferDecision::Skip => {
                    return Ok(task(
                        TransferDirection::Download,
                        &remote_path,
                        &local_path,
                        TransferState::Completed,
                    ))
                }
            };
            let mut remote = sftp
                .open(Path::new(&remote_path))
                .map_err(|error| AppError::Sftp(format!("open remote file failed: {error}")))?;
            let mut bytes = Vec::new();
            remote
                .read_to_end(&mut bytes)
                .map_err(|error| AppError::Sftp(format!("read remote file failed: {error}")))?;
            let mut local = File::create(&destination)
                .map_err(|error| AppError::Io(format!("create local file failed: {error}")))?;
            local
                .write_all(&bytes)
                .map_err(|error| AppError::Io(format!("write local file failed: {error}")))?;
            Ok(task(
                TransferDirection::Download,
                &remote_path,
                &destination,
                TransferState::Completed,
            ))
        })
        .await
        .map_err(|error| AppError::Sftp(format!("download task failed: {error}")))?
    }

    async fn rename(&self, config: SshSessionConfig, from: &str, to: &str) -> Result<()> {
        let from = from.to_string();
        let to = to.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?
                .rename(Path::new(&from), Path::new(&to), None)
                .map_err(|error| AppError::Sftp(format!("rename failed: {error}")))
        })
        .await
        .map_err(|error| AppError::Sftp(format!("rename task failed: {error}")))?
    }

    async fn mkdir(&self, config: SshSessionConfig, path: &str) -> Result<()> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?
                .mkdir(Path::new(&path), 0o755)
                .map_err(|error| AppError::Sftp(format!("mkdir failed: {error}")))
        })
        .await
        .map_err(|error| AppError::Sftp(format!("mkdir task failed: {error}")))?
    }

    async fn delete(&self, config: SshSessionConfig, path: &str) -> Result<()> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let session = linkqinshell_ssh_client::authenticated_session_for_sftp(&config)?;
            let sftp = session
                .sftp()
                .map_err(|error| AppError::Sftp(format!("open sftp failed: {error}")))?;
            sftp.unlink(Path::new(&path))
                .or_else(|_| sftp.rmdir(Path::new(&path)))
                .map_err(|error| AppError::Sftp(format!("delete failed: {error}")))
        })
        .await
        .map_err(|error| AppError::Sftp(format!("delete task failed: {error}")))?
    }
}

fn task(
    direction: TransferDirection,
    source: &str,
    destination: &str,
    state: TransferState,
) -> TransferTask {
    TransferTask {
        id: Uuid::new_v4(),
        direction,
        source: source.into(),
        destination: destination.into(),
        state,
        progress: 1.0,
        error: None,
        started_at: None,
        finished_at: None,
    }
}

fn is_dir(perm: Option<u32>) -> bool {
    const S_IFMT: u32 = 0o170000;
    const S_IFDIR: u32 = 0o040000;
    matches!(perm, Some(value) if value & S_IFMT == S_IFDIR)
}
