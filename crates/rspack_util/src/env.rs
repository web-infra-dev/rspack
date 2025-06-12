use std::sync::LazyLock;

// generate query
static RSPACK_QUERY: LazyLock<Option<String>> =
  LazyLock::new(|| std::env::var("RSPACK_QUERY").ok());

pub fn has_query() -> bool {
  RSPACK_QUERY.is_some()
}
