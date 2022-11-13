pub fn generate_web_rspack_require() -> String {
  include_str!("runtime/web/_rspack_require.js").to_string()
}

pub fn generate_web_rspack_register() -> String {
  include_str!("runtime/web/_rspack_register.js").to_string()
}

pub fn generate_web_dynamic_require() -> String {
  include_str!("runtime/web/_dynamic_require.js").to_string()
}

pub fn generate_web_dynamic_get_chunk_url(has_hash: bool) -> String {
  include_str!("runtime/web/_dynamic_get_chunk_url.js")
    .to_string()
    .replace(
      "__GET_DYNAMIC_URL_HASH_PLACEHOLDER__",
      if has_hash {
        r#"'.' + this.chunkHashData[type][chunkId]"#
      } else {
        r#""""#
      },
    )
}

pub fn generate_web_dynamic_load_script() -> String {
  include_str!("runtime/web/_dynamic_load_script.js").to_string()
}

pub fn generate_web_dynamic_load_style() -> String {
  include_str!("runtime/web/_dynamic_load_style.js").to_string()
}

pub fn generate_web_hot() -> String {
  include_str!("runtime/web/_hot.js").to_string()
}

pub fn generate_web_jsonp() -> String {
  include_str!("runtime/web/_jsonp.js").to_string()
}

pub fn generate_web_load_script_content() -> String {
  include_str!("runtime/web/_load_script_content.js").to_string()
}
