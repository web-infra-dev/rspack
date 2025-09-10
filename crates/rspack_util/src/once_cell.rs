use std::sync::{Mutex, OnceLock};

static LOCK: Mutex<()> = Mutex::new(());

/// Alternative to OnceLock.get_or_try_init
///
/// TODO remove this method after OnceLock.get_or_try_init stable
pub fn once_lock_get_or_try_init<T, E, F>(cell: &OnceLock<T>, init: F) -> Result<&T, E>
where
  F: FnOnce() -> Result<T, E>,
{
  if let Some(value) = cell.get() {
    return Ok(value);
  }

  let _guard = LOCK.lock().expect("should get lock success");
  // Double-check after acquiring the lock
  if let Some(value) = cell.get() {
    return Ok(value);
  }

  let new_value = init()?;
  Ok(cell.get_or_init(|| new_value))
}
