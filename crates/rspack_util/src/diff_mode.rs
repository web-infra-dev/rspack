use once_cell::sync::Lazy;

static IS_DIFF_MODE: Lazy<String> =
  Lazy::new(|| std::env::var("RSPACK_DIFF").ok().unwrap_or_default());

pub fn is_diff_mode() -> bool {
  *IS_DIFF_MODE == "true"
}
