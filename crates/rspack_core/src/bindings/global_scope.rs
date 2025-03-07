use std::{cell::Cell, ptr};

use napi::{bindgen_prelude::Object, Env};
use napi_derive::module_exports;

thread_local!(static CONTEXT: Cell<Env> = Cell::new(Env::from_raw(ptr::null_mut())));

pub struct GlobalScope;

impl GlobalScope {
  pub(crate) fn get_env() -> Env {
    CONTEXT.with(|context| context.get())
  }
}

#[module_exports]
fn init(_exports: Object, env: Env) -> napi::Result<()> {
  CONTEXT.set(env);
  Ok(())
}
