use rspack_core::RuntimeSourceNode;

pub fn generate_node_init_runtime(namespace: &str) -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/node/_init_runtime.js").replace("__rspack_runtime__", namespace),
  }
}

pub fn generate_node_rspack_require() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/node/_rspack_require.js").to_string(),
  }
}
