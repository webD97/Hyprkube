use rhai::{FuncRegistration, Module};
use tauri_plugin_clipboard_manager::ClipboardExt as _;

pub fn build_module(app: tauri::AppHandle) -> Module {
    let mut clipboard_module = Module::new();

    {
        let app = app.clone();

        FuncRegistration::new("write_text").set_into_module(
            &mut clipboard_module,
            move |data: &str| -> Result<(), Box<rhai::EvalAltResult>> {
                app.clipboard()
                    .write_text(data)
                    .map_err(|e| e.to_string())?;
                Ok(())
            },
        );
    }

    clipboard_module
}
