use anyhow::Context;
use linkqinshell_storage::{app_data_dir, database_path, log_path, open_database};
use tracing_subscriber::EnvFilter;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    init_logging()?;
    let runtime = tokio::runtime::Runtime::new().context("create tokio runtime")?;
    runtime
        .block_on(open_database())
        .context("open LinkQinShell database")?;
    let ui = MainWindow::new().context("create main window")?;
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

    ui.run().context("run UI")?;
    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    std::fs::create_dir_all(app_data_dir()).context("create LinkQinShell app data dir")?;
    let _log_path = log_path();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}
