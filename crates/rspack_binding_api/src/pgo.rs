#[cfg(rspack_pgo_generate)]
use std::sync::atomic::{AtomicBool, Ordering};

use napi::{Env, Result};

#[cfg(rspack_pgo_generate)]
static REGISTERED: AtomicBool = AtomicBool::new(false);

#[cfg(rspack_pgo_generate)]
unsafe extern "C" {
  fn __llvm_profile_write_file() -> i32;
}

#[cfg(rspack_pgo_generate)]
#[inline(never)]
pub fn register_profile_dump(env: &Env) -> Result<()> {
  if REGISTERED.swap(true, Ordering::AcqRel) {
    return Ok(());
  }

  env.add_env_cleanup_hook((), |_| unsafe {
    __llvm_profile_write_file();
  })?;
  Ok(())
}

#[cfg(not(rspack_pgo_generate))]
#[inline(never)]
pub fn register_profile_dump(_env: &Env) -> Result<()> {
  Ok(())
}

#[cfg(rspack_pgo_generate)]
#[napi(skip_typescript)]
pub fn write_pgo_profile() -> i32 {
  unsafe { __llvm_profile_write_file() }
}
