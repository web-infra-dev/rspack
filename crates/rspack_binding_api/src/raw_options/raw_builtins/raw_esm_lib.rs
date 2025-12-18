use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use rayon::iter::Either;
use rspack_core::CompilerId;
use rspack_error::ToStringResultToRspackResultExt;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_esm_library::EsmLibraryPlugin;
use rspack_plugin_split_chunks::{CacheGroup, SplitChunksPlugin};
use rspack_regex::RspackRegex;

use crate::{
  module::ModuleObject,
  raw_options::{self, RawSplitChunksOptions},
};

pub type RawNameGetter = Either<String, ThreadsafeFunction<ModuleObject, Option<String>>>;

pub type RawCacheGroupTest =
  Either3<String, RspackRegex, ThreadsafeFunction<ModuleObject, Option<bool>>>;

#[napi(object, object_to_js = false)]
pub struct RawEsmLibraryPlugin<'a> {
  pub preserve_modules: Option<String>,
  pub split_chunks: Option<RawSplitChunksOptions<'a>>,
}

impl<'a> From<RawSplitChunksOptions<'a>> for Vec<CacheGroup> {
  fn from(value: RawSplitChunksOptions<'a>) -> Self {
    let mut groups = rspack_plugin_split_chunks::PluginOptions::from(value).cache_groups;

    groups.sort_by(|a, b| a.priority.total_cmp(&b.priority));

    groups
  }
}
