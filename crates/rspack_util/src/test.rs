use std::sync::LazyLock;

static RSPACK_HOT_TEST: LazyLock<String> =
  LazyLock::new(|| std::env::var("RSPACK_HOT_TEST").ok().unwrap_or_default());

pub fn is_hot_test() -> bool {
  *RSPACK_HOT_TEST == "true"
}
