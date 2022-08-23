use rspack_core::RuntimeSourceNode;

pub fn generate_web_worker_init_runtime() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web_worker/_init_runtime.js").to_string(),
  }
}
