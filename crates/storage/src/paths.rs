use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("LinkQinShell")
}

pub fn database_path() -> PathBuf {
    app_data_dir().join("linkqinshell.db")
}

pub fn log_path() -> PathBuf {
    app_data_dir().join("app.log")
}
