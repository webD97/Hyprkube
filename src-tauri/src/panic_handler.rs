use tauri::Emitter as _;

use crate::frontend_types::BackendPanic;

pub fn setup(app: tauri::AppHandle) {
    use std::panic;

    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        default_hook(info);

        let panic_msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned());

        let frontend_panic_info = BackendPanic {
            thread: std::thread::current().name().map(|s| s.to_owned()),
            location: info.location().map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column(),
                )
            }),
            message: panic_msg,
        };

        if let Err(e) = app.emit("background_task_panic", frontend_panic_info) {
            eprintln!("Failed to emit panic event to frontend: {e}");
        }
    }));
}
