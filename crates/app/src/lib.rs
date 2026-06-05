use linkqinshell_core::{
    AppError, AuthMethod, Connection, ConnectionRepository, Result, SecretStore, SshAuthConfig,
    SshSessionConfig,
};
use linkqinshell_security::MasterPasswordSecretStore;
use linkqinshell_ui_bridge::SaveConnectionRequest;

pub async fn save_connection_request<R>(
    repo: &R,
    request: SaveConnectionRequest,
) -> Result<Connection>
where
    R: ConnectionRepository,
{
    let port = request.validate()?;
    let secret_store = MasterPasswordSecretStore::new(&request.master_password)?;
    let encrypted_password = secret_store
        .encrypt(request.ssh_password.as_bytes())
        .await?;
    let mut connection = Connection::new_ssh(
        request.form.name,
        request.form.host,
        SshAuthConfig {
            method: AuthMethod::Password,
            username: request.form.username,
            encrypted_password: Some(encrypted_password),
            private_key_path: None,
            encrypted_private_key: None,
            encrypted_private_key_passphrase: None,
            remember_secret: true,
        },
    );
    connection.port = port;
    repo.save(&connection).await?;
    Ok(connection)
}

pub async fn connection_sidebar_text<R>(repo: &R) -> Result<String>
where
    R: ConnectionRepository,
{
    let connections = repo.list_active().await?;
    if connections.is_empty() {
        return Ok("No saved connections".into());
    }
    Ok(connections
        .into_iter()
        .map(|connection| {
            format!(
                "{}  {}@{}:{}",
                connection.name, connection.auth.username, connection.host, connection.port
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub async fn first_connection_session_config<R>(
    repo: &R,
    master_password: &str,
) -> Result<SshSessionConfig>
where
    R: ConnectionRepository,
{
    let connection = repo
        .list_active()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Validation("no saved SSH connection is available".into()))?;
    session_config_for_connection(connection, master_password).await
}

pub async fn session_config_for_connection(
    connection: Connection,
    master_password: &str,
) -> Result<SshSessionConfig> {
    let secret_store = MasterPasswordSecretStore::new(master_password)?;
    let password = match &connection.auth.encrypted_password {
        Some(secret) => {
            let plain = secret_store.decrypt(secret).await?;
            Some(String::from_utf8(plain).map_err(|error| {
                AppError::Security(format!("decrypted password is not valid UTF-8: {error}"))
            })?)
        }
        None => None,
    };
    Ok(SshSessionConfig {
        connection,
        password,
        private_key: None,
        private_key_passphrase: None,
    })
}

#[cfg(test)]
mod tests {
    use linkqinshell_core::ConnectionRepository;
    use linkqinshell_storage::{run_migrations, SqliteConnectionRepository};
    use linkqinshell_ui_bridge::{NewConnectionForm, SaveConnectionRequest};
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn saves_password_connection_with_encrypted_secret() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteConnectionRepository::new(pool);
        let request = SaveConnectionRequest {
            master_password: "correct horse battery staple".into(),
            ssh_password: "server-password".into(),
            form: NewConnectionForm {
                name: "prod".into(),
                host: "example.com".into(),
                port: "22".into(),
                username: "root".into(),
                auth_method: "password".into(),
            },
        };

        let saved = crate::save_connection_request(&repo, request)
            .await
            .unwrap();
        let loaded = repo.find_by_id(&saved.id).await.unwrap().unwrap();

        assert_eq!(loaded.name, "prod");
        assert_eq!(loaded.host, "example.com");
        assert_ne!(
            loaded.auth.encrypted_password.as_deref(),
            Some("server-password")
        );
    }

    #[tokio::test]
    async fn formats_saved_connections_for_sidebar_display() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteConnectionRepository::new(pool);
        let request = SaveConnectionRequest {
            master_password: "correct horse battery staple".into(),
            ssh_password: "server-password".into(),
            form: NewConnectionForm {
                name: "prod".into(),
                host: "example.com".into(),
                port: "2222".into(),
                username: "deploy".into(),
                auth_method: "password".into(),
            },
        };

        crate::save_connection_request(&repo, request)
            .await
            .unwrap();
        let text = crate::connection_sidebar_text(&repo).await.unwrap();

        assert!(text.contains("prod"));
        assert!(text.contains("deploy@example.com:2222"));
    }

    #[tokio::test]
    async fn prepares_first_saved_connection_with_decrypted_password() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteConnectionRepository::new(pool);
        let request = SaveConnectionRequest {
            master_password: "correct horse battery staple".into(),
            ssh_password: "server-password".into(),
            form: NewConnectionForm {
                name: "prod".into(),
                host: "example.com".into(),
                port: "22".into(),
                username: "root".into(),
                auth_method: "password".into(),
            },
        };

        crate::save_connection_request(&repo, request)
            .await
            .unwrap();
        let config = crate::first_connection_session_config(&repo, "correct horse battery staple")
            .await
            .unwrap();

        assert_eq!(config.connection.name, "prod");
        assert_eq!(config.password.as_deref(), Some("server-password"));
    }
}
