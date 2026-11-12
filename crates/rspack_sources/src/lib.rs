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
mod content;
mod decoder;
mod dev_tool_code_source;
mod dev_tool_map_source;
mod encoder;
mod error;
mod helpers;
mod line_index;
mod linear_map;
mod original_source;
mod raw_source;
mod replace_source;
mod source;
mod source_content_lines;
mod source_map_source;

/// Feature for rspack persistent cache serialization/deserialization.
#[cfg(feature = "rspack_cacheable")]
pub mod cacheable;

pub use cached_source::CachedSource;
pub use concat_source::ConcatSource;
pub use content::{source_content_eq, update_content_hash};
pub use dev_tool_code_source::DevToolCodeSource;
pub use dev_tool_map_source::DevToolMapSource;
pub use error::{Error, Result};
pub use original_source::OriginalSource;
pub use raw_source::{RawBufferSource, RawStringSource};
pub use replace_source::{ReplaceSource, Replacement, ReplacementEnforce};
pub use source::{
  BoxSource, MapOptions, Mapping, OriginalLocation, Source, SourceExt, SourceMap, SourceValue,
  is_buffer_source,
};
pub use source_map_source::{SourceMapSource, SourceMapSourceOptions, WithoutOriginalOptions};

/// Reexport `StreamChunks` related types.
pub mod stream_chunks {
  pub use super::helpers::{
    Chunks, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks, TextSpan, stream_chunks_default,
  };
}

pub use helpers::{decode_mappings, encode_mappings, utf8_column_to_utf16_column, utf16_len};
