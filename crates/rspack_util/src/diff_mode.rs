use std::sync::LazyLock;

static IS_DIFF_MODE: LazyLock<bool> =
  LazyLock::new(|| std::env::var("RSPACK_DIFF").ok().as_deref() == Some("true"));

pub fn is_diff_mode() -> bool {
  *IS_DIFF_MODE
}
