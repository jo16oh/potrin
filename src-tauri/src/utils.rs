use anyhow::anyhow;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager, Runtime, State};

pub async fn set_rw_state<R: Runtime, T>(app_handle: &AppHandle<R>, value: T) -> anyhow::Result<()>
where
    T: 'static + Sync + Send,
{
    match app_handle.try_state::<RwLock<T>>() {
        Some(ref mut state) => {
            let mut state = state.write().await;
            *state = value;
        }
        None => {
            app_handle.manage(RwLock::new(value));
        }
    };
    Ok(())
}

pub fn get_rw_state<R: Runtime, T>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<State<'_, RwLock<T>>>
where
    T: 'static + Sync + Send,
{
    app_handle
        .try_state::<RwLock<T>>()
        .ok_or(anyhow!("failed to get state"))
}

pub fn get_state<R: Runtime, T>(app_handle: &AppHandle<R>) -> anyhow::Result<&T>
where
    T: 'static + Sync + Send,
{
    app_handle
        .try_state::<T>()
        .map(|r| r.inner())
        .ok_or(anyhow!("failed to get state"))
}
