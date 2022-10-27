mod helpers;

pub use helpers::*;
use rspack_core::rspack_sources::RawSource;

pub fn generate_commonjs_runtime() -> Vec<RawSource> {
  vec![
    RawSource::from(include_str!("helpers/_interop_require.js").to_string()),
    RawSource::from(include_str!("helpers/_export_star.js").to_string()),
  ]
}
