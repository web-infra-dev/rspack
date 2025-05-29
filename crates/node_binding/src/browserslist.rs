#[napi]
pub fn load_browserslist(input: Option<String>, context: String) -> Option<Vec<String>> {
  rspack_browserslist::load_browserslist(input.as_deref(), context.as_str())
}
