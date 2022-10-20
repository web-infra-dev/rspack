mod options;
pub use options::*;

#[cfg(feature = "node-api")]
pub mod threadsafe_function;

thread_local! {
  // Safety: A single node process always share the same napi_env, so it's safe to use a thread local
  #[cfg(feature = "node-api")]
  pub static NAPI_ENV: std::cell::RefCell<Option<napi_sys::napi_env>>  = Default::default();
}
