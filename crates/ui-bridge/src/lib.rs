use linkqinshell_core::{AppError, Connection, Result, SessionState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionListItem {
    pub id: String,
    pub name: String,
    pub host_label: String,
    pub favorite: bool,
    pub status_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTabItem {
    pub id: String,
    pub title: String,
    pub state: SessionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewConnectionForm {
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub auth_method: String,
}

impl From<&Connection> for ConnectionListItem {
    fn from(connection: &Connection) -> Self {
        Self {
            id: connection.id.to_string(),
            name: connection.name.clone(),
            host_label: format!(
                "{}@{}:{}",
                connection.auth.username, connection.host, connection.port
            ),
            favorite: connection.favorite,
            status_label: "Saved".into(),
        }
    }
}

impl NewConnectionForm {
    pub fn validate(&self) -> Result<u16> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("connection name is required".into()));
        }
        if self.host.trim().is_empty() {
            return Err(AppError::Validation("host is required".into()));
        }
        if self.username.trim().is_empty() {
            return Err(AppError::Validation("username is required".into()));
        }
        self.port
            .parse::<u16>()
            .map_err(|_| AppError::Validation("port must be a number between 1 and 65535".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_connection_form_rejects_invalid_port() {
        let form = NewConnectionForm {
            name: "prod".into(),
            host: "example.com".into(),
            port: "ssh".into(),
            username: "root".into(),
            auth_method: "password".into(),
        };
        let error = form.validate().unwrap_err().to_string();
        assert!(error.contains("port must be a number"));
    }
}
