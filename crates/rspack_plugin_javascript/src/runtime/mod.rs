mod helpers;

pub use helpers::*;
use rspack_core::rspack_sources::RawSource;

pub fn generate_interop_require() -> RawSource {
  RawSource::from(include_str!("helpers/_interop_require.js").to_string())
}
