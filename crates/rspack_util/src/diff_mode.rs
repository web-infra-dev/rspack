use std::sync::LazyLock;

static IS_DIFF_MODE: LazyLock<String> =
  LazyLock::new(|| std::env::var("RSPACK_DIFF").ok().unwrap_or_default());

pub fn is_diff_mode() -> bool {
  *IS_DIFF_MODE == "true"
}
