use std::{cell::OnceCell, sync::OnceLock};

/// Alternative to OnceCell.get_or_try_init
///
/// TODO remove this method after OnceCell.get_or_try_init stable
pub fn once_cell_get_or_try_init<T, E, F>(cell: &OnceCell<T>, init: F) -> Result<&T, E>
where
  F: FnOnce() -> Result<T, E>,
{
  if let Some(value) = cell.get() {
    return Ok(value);
  }
  let new_value = init()?;
  if cell.set(new_value).is_err() {
    unreachable!()
  }
  let Some(value) = cell.get() else {
    unreachable!()
  };
  Ok(value)
}

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
  let new_value = init()?;
  if cell.set(new_value).is_err() {
    unreachable!()
  }
  let Some(value) = cell.get() else {
    unreachable!()
  };
  Ok(value)
}
