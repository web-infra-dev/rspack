use std::collections::HashMap;

pub type Define = HashMap<String, String>;

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  pub minify: bool,
  pub polyfill: bool,
  pub browserslist: Vec<String>,
  pub define: Define,
}
