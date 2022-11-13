pub fn generate_node_init_runtime(namespace: &str) -> String {
  include_str!("runtime/node/_init_runtime.js").replace("__rspack_runtime__", namespace)
}

pub fn generate_node_rspack_require() -> String {
  include_str!("runtime/node/_rspack_require.js").to_string()
}

pub fn generate_node_dynamic_require() -> String {
  include_str!("runtime/node/_dynamic_require.js").to_string()
}

pub fn generate_node_load_chunk() -> String {
  include_str!("runtime/node/_dynamic_load_chunk.js").to_string()
}

pub fn generate_node_dynamic_get_chunk_url(has_hash: bool) -> String {
  include_str!("runtime/node/_dynamic_get_chunk_url.js").replace(
    "__GET_DYNAMIC_URL_HASH_PLACEHOLDER__",
    if has_hash {
      r#"'.' + this.chunkHashData[type][chunkId]"#
    } else {
      r#""""#
    },
  )
}
