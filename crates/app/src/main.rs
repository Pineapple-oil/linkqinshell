use anyhow::Context;
use linkqinshell_app::{
    connection_sidebar_text, first_connection_session_config, save_connection_request,
};
use linkqinshell_core::{ConnectionRepository, SshBackend};
use linkqinshell_ssh_client::Ssh2Backend;
use linkqinshell_storage::{
    app_data_dir, database_path, log_path, open_database, SqliteConnectionRepository,
};
use linkqinshell_ui_bridge::{NewConnectionForm, SaveConnectionRequest};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    init_logging()?;
    let runtime = tokio::runtime::Runtime::new().context("create tokio runtime")?;
    let pool = runtime
        .block_on(open_database())
        .context("open LinkQinShell database")?;
    let repo = SqliteConnectionRepository::new(pool);
    let ssh_backend = Ssh2Backend::default();
    let ui = MainWindow::new().context("create main window")?;
    set_connection_summary(&ui, &runtime, &repo);
    ui.set_status_text(
        format!(
            "Data: {} | Database: {}",
            app_data_dir().display(),
            database_path().display()
        )
        .into(),
    );

    let weak = ui.as_weak();
    ui.on_new_connection_requested(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_status_text(
                "New connection dialog is part of the P0 workflow; backend validation is ready."
                    .into(),
            );
        }
    });

    let weak = ui.as_weak();
    ui.on_quick_connect_requested(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_status_text(
                "Quick connect will create a temporary SSH session and can be saved after success."
                    .into(),
            );
        }
    });

    let runtime = Arc::new(runtime);
    let repo_for_save = repo.clone();
    let runtime_for_save = runtime.clone();
    let weak = ui.as_weak();
    ui.on_save_connection_requested(
        move |master_password, name, host, port, username, ssh_password| {
            if let Some(ui) = weak.upgrade() {
                let request = SaveConnectionRequest {
                    master_password: master_password.to_string(),
                    ssh_password: ssh_password.to_string(),
                    form: NewConnectionForm {
                        name: name.to_string(),
                        host: host.to_string(),
                        port: port.to_string(),
                        username: username.to_string(),
                        auth_method: "password".into(),
                    },
                };
                match runtime_for_save.block_on(save_connection_request(&repo_for_save, request)) {
                    Ok(connection) => {
                        ui.set_status_text(
                            format!("Saved encrypted SSH connection: {}", connection.name).into(),
                        );
                        set_connection_summary(&ui, &runtime_for_save, &repo_for_save);
                    }
                    Err(error) => {
                        ui.set_status_text(format!("Save failed: {error}").into());
                    }
                }
            }
        },
    );

    let repo_for_connect = repo.clone();
    let ssh_for_connect = ssh_backend.clone();
    let runtime_for_connect = runtime.clone();
    let weak = ui.as_weak();
    ui.on_connect_first_requested(move |master_password| {
        if let Some(ui) = weak.upgrade() {
            let result = runtime_for_connect.block_on(async {
                let config =
                    first_connection_session_config(&repo_for_connect, &master_password).await?;
                ssh_for_connect.connect(config).await
            });
            match result {
                Ok(handle) => {
                    ui.set_status_text(format!("SSH connected. Session: {}", handle.id).into());
                }
                Err(error) => {
                    ui.set_status_text(format!("SSH connect failed: {error}").into());
                }
            }
        }
    });

    ui.run().context("run UI")?;
    Ok(())
}

fn set_connection_summary(
    ui: &MainWindow,
    runtime: &tokio::runtime::Runtime,
    repo: &SqliteConnectionRepository,
) {
    match runtime.block_on(repo.list_active()) {
        Ok(connections) if connections.is_empty() => {
            ui.set_connection_summary("No saved connections".into());
            ui.set_connection_list_text("Create and save an SSH connection below.".into());
        }
        Ok(connections) => {
            ui.set_connection_summary(format!("{} saved connection(s)", connections.len()).into());
            match runtime.block_on(connection_sidebar_text(repo)) {
                Ok(text) => ui.set_connection_list_text(text.into()),
                Err(error) => ui.set_connection_list_text(format!("List error: {error}").into()),
            }
        }
        Err(error) => {
            ui.set_connection_summary(format!("Connection list unavailable: {error}").into());
            ui.set_connection_list_text("Storage unavailable".into());
        }
    }
}

fn init_logging() -> anyhow::Result<()> {
    std::fs::create_dir_all(app_data_dir()).context("create LinkQinShell app data dir")?;
    let _log_path = log_path();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}
