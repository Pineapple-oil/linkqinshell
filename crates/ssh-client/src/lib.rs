use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use linkqinshell_core::{
    AppError, AuthMethod, HostKey, Result, SessionHandle, SessionState, SshBackend,
    SshSessionConfig,
};
use sha2::{Digest, Sha256};
use ssh2::Session;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Ssh2Backend {
    sessions: Arc<Mutex<HashMap<Uuid, SessionState>>>,
}

#[async_trait]
impl SshBackend for Ssh2Backend {
    async fn connect(&self, config: SshSessionConfig) -> Result<SessionHandle> {
        let id = Uuid::new_v4();
        set_state(&self.sessions, id, SessionState::Connecting)?;
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || {
            set_state(&sessions, id, SessionState::Authenticating)?;
            let session = authenticated_session(&config)?;
            let mut channel = session
                .channel_session()
                .map_err(|error| AppError::Ssh(format!("open channel failed: {error}")))?;
            channel
                .request_pty(
                    &config.connection.terminal_type,
                    None,
                    Some((100, 30, 0, 0)),
                )
                .map_err(|error| AppError::Ssh(format!("request pty failed: {error}")))?;
            channel
                .shell()
                .map_err(|error| AppError::Ssh(format!("open shell failed: {error}")))?;
            let mut initial = [0_u8; 1024];
            let _ = channel.read(&mut initial);
            set_state(&sessions, id, SessionState::Connected)?;
            Ok(SessionHandle {
                id,
                state: SessionState::Connected,
            })
        })
        .await
        .map_err(|error| AppError::Ssh(format!("ssh task failed: {error}")))?
    }

    async fn verify_host_key(&self, host: &str, port: u16) -> Result<HostKey> {
        let host = host.to_string();
        tokio::task::spawn_blocking(move || observe_host_key(&host, port))
            .await
            .map_err(|error| AppError::Ssh(format!("host key task failed: {error}")))?
    }

    async fn write_stdin(&self, session_id: Uuid, _bytes: &[u8]) -> Result<()> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::Ssh("session state lock poisoned".into()))?;
        if sessions.contains_key(&session_id) {
            Ok(())
        } else {
            Err(AppError::Ssh("session not found".into()))
        }
    }

    async fn disconnect(&self, session_id: Uuid) -> Result<()> {
        set_state(&self.sessions, session_id, SessionState::Closed)
    }
}

pub fn authenticated_session_for_sftp(config: &SshSessionConfig) -> Result<Session> {
    authenticated_session(config)
}

fn authenticated_session(config: &SshSessionConfig) -> Result<Session> {
    let tcp = TcpStream::connect((config.connection.host.as_str(), config.connection.port))
        .map_err(|error| AppError::Ssh(format!("tcp connect failed: {error}")))?;
    tcp.set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|error| AppError::Ssh(format!("set timeout failed: {error}")))?;
    let mut session =
        Session::new().map_err(|error| AppError::Ssh(format!("session init failed: {error}")))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|error| AppError::Ssh(format!("handshake failed: {error}")))?;

    match config.connection.auth.method {
        AuthMethod::Password => {
            let password = config
                .password
                .as_ref()
                .ok_or_else(|| AppError::Authentication("password is required".into()))?;
            session
                .userauth_password(&config.connection.auth.username, password)
                .map_err(|error| {
                    AppError::Authentication(format!("password auth failed: {error}"))
                })?;
        }
        AuthMethod::PrivateKey => {
            let path = config
                .connection
                .auth
                .private_key_path
                .as_ref()
                .ok_or_else(|| AppError::Authentication("private key path is required".into()))?;
            session
                .userauth_pubkey_file(
                    &config.connection.auth.username,
                    None,
                    Path::new(path),
                    config.private_key_passphrase.as_deref(),
                )
                .map_err(|error| {
                    AppError::Authentication(format!("private key auth failed: {error}"))
                })?;
        }
    }

    if session.authenticated() {
        Ok(session)
    } else {
        Err(AppError::Authentication(
            "ssh authentication was not accepted".into(),
        ))
    }
}

fn observe_host_key(host: &str, port: u16) -> Result<HostKey> {
    let tcp = TcpStream::connect((host, port))
        .map_err(|error| AppError::Ssh(format!("tcp connect failed: {error}")))?;
    let mut session =
        Session::new().map_err(|error| AppError::Ssh(format!("session init failed: {error}")))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|error| AppError::Ssh(format!("handshake failed: {error}")))?;
    let (key, key_type) = session
        .host_key()
        .ok_or_else(|| AppError::HostKey("server did not provide a host key".into()))?;
    let fingerprint = format!("SHA256:{}", base64_sha256(key));
    let now = Utc::now();
    Ok(HostKey {
        id: Uuid::new_v4(),
        host: host.to_string(),
        port,
        key_type: format!("{key_type:?}"),
        fingerprint_sha256: fingerprint,
        fingerprint_md5: None,
        public_key: base64_bytes(key),
        first_seen_at: now,
        last_seen_at: now,
        trusted: false,
    })
}

fn base64_sha256(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let digest = Sha256::digest(bytes);
    STANDARD.encode(digest)
}

fn base64_bytes(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.encode(bytes)
}

fn set_state(
    sessions: &Arc<Mutex<HashMap<Uuid, SessionState>>>,
    id: Uuid,
    state: SessionState,
) -> Result<()> {
    sessions
        .lock()
        .map_err(|_| AppError::Ssh("session state lock poisoned".into()))?
        .insert(id, state);
    Ok(())
}
