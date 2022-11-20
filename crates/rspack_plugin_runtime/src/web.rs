pub fn generate_web_hot() -> String {
  include_str!("runtime/web/_hot.js").to_string()
}

pub fn generate_web_jsonp() -> String {
  include_str!("runtime/web/_jsonp.js").to_string()
}
