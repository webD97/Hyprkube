use tracing::info;

#[tauri::command]
#[tracing::instrument(target = "crate::frontend", skip_all)]
pub fn log_stdout(line: &str) {
    info!("{line}");
}
