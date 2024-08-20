use std::sync::OnceLock;
use anyhow::anyhow;

pub fn set_once_lock<T>(lock: &OnceLock<T>, value: T) -> anyhow::Result<()> {
    lock.set(value)
        .map_err(|_| anyhow!("Failed to set value to OnceLock"))?;
    Ok(())
}

pub fn get_once_lock<T>(lock: &OnceLock<T>) -> anyhow::Result<&T> {
    let result = lock
        .get()
        .ok_or_else(|| anyhow!("Failed to get value of OnceLock"))?;
    Ok(result)
}
