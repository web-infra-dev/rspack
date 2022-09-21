use rspack_core::rspack_sources::RawSource;

pub fn generate_web_worker_init_runtime(namespace: &str) -> RawSource {
  RawSource::from(
    include_str!("runtime/web_worker/_init_runtime.js")
      .to_string()
      .replace("__rspack_runtime__", namespace),
  )
}
