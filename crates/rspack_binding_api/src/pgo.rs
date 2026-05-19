use std::sync::atomic::{AtomicBool, Ordering};

use napi::{Env, Result};

static REGISTERED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
  fn __llvm_profile_write_file() -> i32;
}

pub fn register_profile_dump(env: &Env) -> Result<()> {
  if REGISTERED.swap(true, Ordering::AcqRel) {
    return Ok(());
  }

  env.add_env_cleanup_hook((), |_| unsafe {
    __llvm_profile_write_file();
  })?;
  Ok(())
}

#[napi(skip_typescript)]
pub fn write_pgo_profile() -> i32 {
  unsafe { __llvm_profile_write_file() }
}
