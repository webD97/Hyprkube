pub trait ManagerExt {
    fn state<S: ManagedState>(&self) -> tauri::State<'_, S::WrappedState>;
}

impl ManagerExt for tauri::AppHandle {
    fn state<S: ManagedState>(&self) -> tauri::State<'_, S::WrappedState> {
        tauri::Manager::state::<S::WrappedState>(self)
    }
}

pub trait ManagedState {
    type WrappedState: Send + Sync + 'static;

    fn build(app: tauri::AppHandle) -> Self::WrappedState;
}
