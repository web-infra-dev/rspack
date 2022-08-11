use rspack_core::RuntimeSourceNode;

pub fn generate_common_init_runtime() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/common/_init_runtime.js").to_string(),
  }
}

pub fn generate_common_module_and_chunk_data() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/common/_module_and_chunk_data.js").to_string(),
  }
}

pub fn generate_common_check_by_id() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/common/_check_by_id.js").to_string(),
  }
}

pub fn generate_common_public_path(public_path: &str) -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/common/_public_path.js")
      .to_string()
      .replace("__PUBLIC_PATH_PLACEHOLDER__", public_path),
  }
}

pub fn generate_common_dynamic_data() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/common/_dynamic_data.js").to_string(),
  }
}
