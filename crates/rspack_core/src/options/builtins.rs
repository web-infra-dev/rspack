#[derive(Debug, Clone, Default)]
pub struct Builtins {
  pub minify: bool,
  pub polyfill: bool,
  pub browserslist: Vec<String>,
  pub jsx: bool
}
