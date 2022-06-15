use napi::Env;
use napi_derive::napi;
use once_cell::sync::OnceCell;

static CUSTOM_TRACE_SUBSCRIBER: OnceCell<bool> = OnceCell::new();

#[napi]
pub fn init_custom_trace_subscriber(
  mut env: Env,
  // trace_out_file_path: Option<String>,
) -> napi::Result<()> {
  CUSTOM_TRACE_SUBSCRIBER.get_or_init(|| {
    let guard = rspack_core::log::enable_tracing_by_env_with_chrome_layer();
    if let Some(guard) = guard {
      env
        .add_env_cleanup_hook(guard, |flush_guard| {
          flush_guard.flush();
          drop(flush_guard);
        })
        .expect("Should able to initialize cleanup for custom trace subscriber");
    }
    true
  });

  Ok(())
}
