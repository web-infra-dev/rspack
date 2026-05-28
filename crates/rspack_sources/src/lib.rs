//! Rusty [`webpack-sources`](https://github.com/webpack/webpack-sources) port.

#![allow(
  clippy::collapsible_if,
  clippy::get_unwrap,
  clippy::let_and_return,
  clippy::manual_string_new,
  clippy::map_unwrap_or,
  clippy::rc_buffer,
  clippy::redundant_clone,
  clippy::rest_pat_in_fully_bound_structs,
  clippy::trivial_regex,
  clippy::uninlined_format_args,
  clippy::unreadable_literal,
  clippy::unwrap_used
)]

mod cached_source;
mod concat_source;
mod decoder;
mod encoder;
mod error;
mod helpers;
mod linear_map;
mod object_pool;
mod original_source;
mod raw_source;
mod replace_source;
mod source;
mod source_content_lines;
mod source_map_source;
mod with_utf16;

/// Feature for rspack persistent cache serialization/deserialization.
#[cfg(feature = "rspack_cacheable")]
pub mod cacheable;

pub use cached_source::CachedSource;
pub use concat_source::ConcatSource;
pub use error::{Error, Result};
pub use original_source::OriginalSource;
pub use raw_source::{RawBufferSource, RawStringSource};
pub use replace_source::{ReplaceSource, Replacement, ReplacementEnforce};
pub use source::{
  BoxSource, MapOptions, Mapping, OriginalLocation, Source, SourceExt, SourceMap, SourceValue,
};
pub use source_map_source::{SourceMapSource, SourceMapSourceOptions, WithoutOriginalOptions};

/// Reexport `StreamChunks` related types.
pub mod stream_chunks {
  pub use super::helpers::{
    Chunks, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks, TextSpan, stream_chunks_default,
  };
}

pub use helpers::{decode_mappings, encode_mappings, utf16_len};
pub use object_pool::ObjectPool;
