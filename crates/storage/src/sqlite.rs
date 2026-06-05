use async_trait::async_trait;
use chrono::Utc;
use linkqinshell_core::{
    AppError, Connection, ConnectionId, ConnectionRepository, HostKey, HostKeyDecision,
    HostKeyRepository, Result,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::database_path;

pub async fn open_database() -> Result<SqlitePool> {
    let path = database_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| AppError::Io(format!("create data directory failed: {error}")))?;
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|error| AppError::Storage(format!("open database failed: {error}")))?;
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    for statement in include_str!("../../../migrations/0001_p0_schema.sql").split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        sqlx::query(statement)
            .execute(pool)
            .await
            .map_err(|error| AppError::Storage(format!("migration failed: {error}")))?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct SqliteConnectionRepository {
    pool: SqlitePool,
}

impl SqliteConnectionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(Clone)]
pub struct SqliteHostKeyRepository {
    pool: SqlitePool,
}

impl SqliteHostKeyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConnectionRepository for SqliteConnectionRepository {
    async fn save(&self, connection: &Connection) -> Result<()> {
        connection.validate()?;
        let payload = serde_json::to_string(connection)
            .map_err(|error| AppError::Storage(format!("serialize connection failed: {error}")))?;
        sqlx::query(
            r#"
            INSERT INTO connections (
              id, name, host, port, username, favorite, recent_at,
              deleted_at, recover_until, payload, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
              name=excluded.name,
              host=excluded.host,
              port=excluded.port,
              username=excluded.username,
              favorite=excluded.favorite,
              recent_at=excluded.recent_at,
              deleted_at=excluded.deleted_at,
              recover_until=excluded.recover_until,
              payload=excluded.payload,
              updated_at=excluded.updated_at
            "#,
        )
        .bind(connection.id.to_string())
        .bind(&connection.name)
        .bind(&connection.host)
        .bind(i64::from(connection.port))
        .bind(&connection.auth.username)
        .bind(connection.favorite)
        .bind(connection.recent_at.map(|d| d.to_rfc3339()))
        .bind(connection.soft_delete.deleted_at.map(|d| d.to_rfc3339()))
        .bind(connection.soft_delete.recover_until.map(|d| d.to_rfc3339()))
        .bind(payload)
        .bind(connection.created_at.to_rfc3339())
        .bind(connection.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Storage(format!("save connection failed: {error}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: &ConnectionId) -> Result<Option<Connection>> {
        let row = sqlx::query("SELECT payload FROM connections WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| AppError::Storage(format!("find connection failed: {error}")))?;
        row.map(|row| decode_connection(row.get::<String, _>("payload")))
            .transpose()
    }

    async fn list_active(&self) -> Result<Vec<Connection>> {
        let rows = sqlx::query(
            "SELECT payload FROM connections WHERE deleted_at IS NULL ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Storage(format!("list connections failed: {error}")))?;
        rows.into_iter()
            .map(|row| decode_connection(row.get::<String, _>("payload")))
            .collect()
    }

    async fn soft_delete(&self, id: &ConnectionId) -> Result<()> {
        let mut connection = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Storage("connection not found".into()))?;
        connection.mark_deleted(Utc::now());
        self.save(&connection).await
    }

    async fn restore(&self, id: &ConnectionId) -> Result<()> {
        let mut connection = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Storage("connection not found".into()))?;
        connection.restore(Utc::now())?;
        self.save(&connection).await
    }
}

fn decode_connection(payload: String) -> Result<Connection> {
    serde_json::from_str(&payload)
        .map_err(|error| AppError::Storage(format!("decode connection failed: {error}")))
}

#[async_trait]
impl HostKeyRepository for SqliteHostKeyRepository {
    async fn find(&self, host: &str, port: u16) -> Result<Option<HostKey>> {
        let row = sqlx::query("SELECT * FROM host_keys WHERE host = ?1 AND port = ?2")
            .bind(host)
            .bind(i64::from(port))
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| AppError::Storage(format!("find host key failed: {error}")))?;
        row.map(decode_host_key).transpose()
    }

    async fn trust(&self, key: &HostKey, decision: HostKeyDecision) -> Result<()> {
        if matches!(decision, HostKeyDecision::Reject) {
            return Err(AppError::HostKey("host key trust was rejected".into()));
        }
        sqlx::query(
            r#"
            INSERT INTO host_keys (
              id, host, port, key_type, fingerprint_sha256, fingerprint_md5,
              public_key, first_seen_at, last_seen_at, trusted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)
            ON CONFLICT(host, port) DO UPDATE SET
              key_type=excluded.key_type,
              fingerprint_sha256=excluded.fingerprint_sha256,
              fingerprint_md5=excluded.fingerprint_md5,
              public_key=excluded.public_key,
              last_seen_at=excluded.last_seen_at,
              trusted=1
            "#,
        )
        .bind(key.id.to_string())
        .bind(&key.host)
        .bind(i64::from(key.port))
        .bind(&key.key_type)
        .bind(&key.fingerprint_sha256)
        .bind(&key.fingerprint_md5)
        .bind(&key.public_key)
        .bind(key.first_seen_at.to_rfc3339())
        .bind(key.last_seen_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Storage(format!("trust host key failed: {error}")))?;
        Ok(())
    }
}

fn decode_host_key(row: sqlx::sqlite::SqliteRow) -> Result<HostKey> {
    let first_seen_at =
        chrono::DateTime::parse_from_rfc3339(row.get::<String, _>("first_seen_at").as_str())
            .map_err(|error| AppError::Storage(format!("decode first_seen_at failed: {error}")))?
            .with_timezone(&Utc);
    let last_seen_at =
        chrono::DateTime::parse_from_rfc3339(row.get::<String, _>("last_seen_at").as_str())
            .map_err(|error| AppError::Storage(format!("decode last_seen_at failed: {error}")))?
            .with_timezone(&Utc);
    Ok(HostKey {
        id: row
            .get::<String, _>("id")
            .parse()
            .map_err(|error| AppError::Storage(format!("decode host key id failed: {error}")))?,
        host: row.get("host"),
        port: row.get::<i64, _>("port") as u16,
        key_type: row.get("key_type"),
        fingerprint_sha256: row.get("fingerprint_sha256"),
        fingerprint_md5: row.get("fingerprint_md5"),
        public_key: row.get("public_key"),
        first_seen_at,
        last_seen_at,
        trusted: row.get::<bool, _>("trusted"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use linkqinshell_core::{AuthMethod, SshAuthConfig};

    fn auth() -> SshAuthConfig {
        SshAuthConfig {
            method: AuthMethod::Password,
            username: "root".into(),
            encrypted_password: Some("ciphertext".into()),
            private_key_path: None,
            encrypted_private_key: None,
            encrypted_private_key_passphrase: None,
            remember_secret: true,
        }
    }

    #[tokio::test]
    async fn migrations_create_connections_table_and_repository_round_trips() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteConnectionRepository::new(pool);
        let connection = Connection::new_ssh("prod", "example.com", auth());

        repo.save(&connection).await.unwrap();
        let loaded = repo.find_by_id(&connection.id).await.unwrap().unwrap();
        assert_eq!(loaded.name, "prod");
        assert_eq!(repo.list_active().await.unwrap().len(), 1);

        repo.soft_delete(&connection.id).await.unwrap();
        assert!(repo.list_active().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn host_key_repository_trusts_and_loads_key() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteHostKeyRepository::new(pool);
        let now = Utc::now();
        let key = HostKey {
            id: uuid::Uuid::new_v4(),
            host: "example.com".into(),
            port: 22,
            key_type: "ssh-ed25519".into(),
            fingerprint_sha256: "SHA256:test".into(),
            fingerprint_md5: None,
            public_key: "public".into(),
            first_seen_at: now,
            last_seen_at: now,
            trusted: false,
        };

        repo.trust(&key, HostKeyDecision::TrustNew).await.unwrap();
        let loaded = repo.find("example.com", 22).await.unwrap().unwrap();
        assert!(loaded.trusted);
        assert_eq!(loaded.fingerprint_sha256, "SHA256:test");
    }
}
