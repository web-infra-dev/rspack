use std::cell::Cell;
use std::fmt;

std::thread_local!(static ENTERED: Cell<bool> = Cell::new(false));

pub struct Enter {
  _priv: (),
}

pub struct EnterError {
  _priv: (),
}

impl fmt::Debug for EnterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("EnterError").finish()
  }
}

impl fmt::Display for EnterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "an execution scope has already been entered")
  }
}

impl std::error::Error for EnterError {}

pub fn enter() -> Result<Enter, EnterError> {
  ENTERED.with(|c| {
    if c.get() {
      Err(EnterError { _priv: () })
    } else {
      c.set(true);

      Ok(Enter { _priv: () })
    }
  })
}

impl fmt::Debug for Enter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Enter").finish()
  }
}

impl Drop for Enter {
  fn drop(&mut self) {
    ENTERED.with(|c| {
      assert!(c.get());
      c.set(false);
    });
  }
}
