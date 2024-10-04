use anyhow::anyhow;
use std::sync::OnceLock;

pub fn set_once_lock<T>(lock: &OnceLock<T>, value: T) -> anyhow::Result<()> {
    lock.set(value)
        .map_err(|_| anyhow!("Failed to set value to OnceLock"))
}

pub fn get_once_lock<T>(lock: &OnceLock<T>) -> anyhow::Result<&T> {
    lock.get().ok_or(anyhow!("Failed to get value of OnceLock"))
}
