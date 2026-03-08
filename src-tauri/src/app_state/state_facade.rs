pub trait StateFacade {
    fn state<S: ManagedState>(&self) -> tauri::State<'_, S::WrappedState>;
}

pub trait ManagedState {
    type WrappedState: Send + Sync + 'static;

    fn build(app: tauri::AppHandle) -> Self::WrappedState;
}

impl StateFacade for tauri::AppHandle {
    fn state<S: ManagedState>(&self) -> tauri::State<'_, S::WrappedState> {
        tauri::Manager::state::<S::WrappedState>(self)
    }
}
