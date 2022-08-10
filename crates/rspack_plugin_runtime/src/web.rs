use rspack_core::RuntimeSourceNode;

pub fn generate_web_rspack_require() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_rspack_require.js").to_string(),
  }
}

pub fn generate_web_rspack_register() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_rspack_register.js").to_string(),
  }
}

pub fn generate_web_dynamic_require() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_dynamic_require.js").to_string(),
  }
}

pub fn generate_web_dynamic_get_chunk_url() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_dynamic_get_chunk_url.js").to_string(),
  }
}

pub fn generate_web_dynamic_load_script() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_dynamic_load_script.js").to_string(),
  }
}

pub fn generate_web_dynamic_load_style() -> RuntimeSourceNode {
  RuntimeSourceNode {
    content: include_str!("runtime/web/_dynamic_load_style.js").to_string(),
  }
}
