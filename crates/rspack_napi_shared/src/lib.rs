mod errors;
pub use errors::{Error, NapiErrorExt, NapiResultExt, Result, RspackErrorExt, RspackResultExt};

pub mod threadsafe_function;

thread_local! {
  // Safety: A single node process always share the same napi_env, so it's safe to use a thread local
  pub static NAPI_ENV: std::cell::RefCell<Option<napi::sys::napi_env>>  = Default::default();
}
