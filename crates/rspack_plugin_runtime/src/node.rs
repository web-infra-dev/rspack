use rspack_core::rspack_sources::RawSource;

pub fn generate_node_init_runtime(namespace: &str) -> RawSource {
  RawSource::from(
    include_str!("runtime/node/_init_runtime.js").replace("__rspack_runtime__", namespace),
  )
}

pub fn generate_node_rspack_require() -> RawSource {
  RawSource::from(include_str!("runtime/node/_rspack_require.js").to_string())
}
