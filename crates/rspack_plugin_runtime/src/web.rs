#[macro_export]
macro_rules! generate_web_rspack_require {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_rspack_require.js").to_string(),
    }
  }};
}

#[macro_export]
macro_rules! generate_web_rspack_register {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_rspack_register.js").to_string(),
    }
  }};
}

#[macro_export]
macro_rules! generate_web_dynamic_require {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_dynamic_require.js").to_string(),
    }
  }};
}

#[macro_export]
macro_rules! generate_web_dynamic_get_chunk_url {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_dynamic_get_chunk_url.js").to_string(),
    }
  }};
}

#[macro_export]
macro_rules! generate_web_dynamic_load_script {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_dynamic_load_script.js").to_string(),
    }
  }};
}

#[macro_export]
macro_rules! generate_web_dynamic_load_style {
  () => {{
    RuntimeSourceNode {
      content: include_str!("runtime/web/_dynamic_load_style.js").to_string(),
    }
  }};
}
