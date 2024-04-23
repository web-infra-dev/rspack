use once_cell::sync::Lazy;

static RSPACK_HOT_TEST: Lazy<String> =
  Lazy::new(|| std::env::var("RSPACK_HOT_TEST").ok().unwrap_or_default());

pub fn is_hot_test() -> bool {
  *RSPACK_HOT_TEST == "true"
}
