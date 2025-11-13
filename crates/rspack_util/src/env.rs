use std::sync::LazyLock;

// generate query
static RSPACK_QUERY: LazyLock<Option<String>> =
  LazyLock::new(|| std::env::var("RSPACK_QUERY").ok());

pub fn has_query() -> bool {
  RSPACK_QUERY.is_some()
}

// debug memory statistics
static RSPACK_DEBUG_MEMORY: LazyLock<Option<String>> =
  LazyLock::new(|| std::env::var("RSPACK_DEBUG_MEMORY").ok());

pub fn should_debug_memory() -> bool {
  RSPACK_DEBUG_MEMORY.is_some()
}
