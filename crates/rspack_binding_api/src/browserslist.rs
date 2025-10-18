#[napi]
pub fn load_browserslist(
  input: Option<String>,
  context: String,
) -> Result<Vec<String>, napi::Error> {
  rspack_browserslist::load_browserslist(input.as_deref(), context.as_str())
    .map_err(|e| napi::Error::from_reason(format!("Failed to load browserslist: {}", e)))
}
