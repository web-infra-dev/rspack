use std::sync::LazyLock;

// contain all tracing info and used for detailed analysis for rspack developers
pub static TRACING_ALL_PRESET: &'static str = "trace";
// contain tracing info useful for rspack users
pub static TRACING_OVERVIEW_PRESET: &'static str = "info";

// only contain most important tracing info useful for rspack benchmark, don't add too much noise here
pub static TRACING_BENCH_PRESET: LazyLock<String> =
  LazyLock::new(|| format!("off,{TRACING_BENCH_TARGET}=info"));

pub static TRACING_BENCH_TARGET: &'static str = "rspack_compilation_main";
