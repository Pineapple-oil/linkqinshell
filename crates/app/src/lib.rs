use linkqinshell_core::{
    AuthMethod, Connection, ConnectionRepository, Result, SecretStore, SshAuthConfig,
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
}
