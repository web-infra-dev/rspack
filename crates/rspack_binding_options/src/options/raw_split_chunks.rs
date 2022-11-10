#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::CompilerOptionsBuilder;
use rspack_plugin_split_chunks::{CacheGroupOptions, ChunkType, SizeType, SplitChunksOptions};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use crate::RawOption;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawSplitChunksOptions {
  pub cache_groups: HashMap<String, RawCacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub default_size_types: Option<Vec<SizeType>>,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawSplitChunksOptions {
  pub cache_groups: HashMap<String, RawCacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub default_size_types: Option<Vec<SizeType>>,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

impl RawOption<SplitChunksOptions> for RawSplitChunksOptions {
  #[allow(clippy::field_reassign_with_default)]
  fn to_compiler_option(
    self,
    options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<SplitChunksOptions> {
    let mut defaults = SplitChunksOptions::default();
    // TODO: Supports css
    let is_enable_css = false;
    let is_production = matches!(options.mode, Some(rspack_core::Mode::Production));
    let is_development = !is_production;
    defaults.default_size_types = Some(if is_enable_css {
      vec![SizeType::JavaScript, SizeType::Css, SizeType::Unknown]
    } else {
      vec![SizeType::JavaScript, SizeType::Unknown]
    });
    defaults.chunks = Some(ChunkType::Async);
    defaults.min_chunks = 1;
    defaults.min_size = if is_production { 20000 } else { 10000 };
    defaults.min_remaining_size = if is_development { Some(0) } else { None };
    defaults.enforce_size_threshold = Some(if is_production { 50000 } else { 30000 });
    defaults.max_async_requests = Some(if is_production { 30 } else { usize::MAX });
    defaults.max_initial_requests = Some(if is_production { 30 } else { usize::MAX });
    defaults.automatic_name_delimiter = Some("-".to_string());

    defaults.cache_groups.extend(From::from([
      (
        "default".to_string(),
        CacheGroupOptions {
          min_chunks: 2.into(),
          priority: -20.into(),
          id_hint: "".to_string().into(),
          ..Default::default()
        },
      ),
      (
        "defaultVendors".to_string(),
        CacheGroupOptions {
          id_hint: "vendors".to_string().into(),
          reuse_existing_chunk: true.into(),
          test: Some(
            Arc::new(|module| {
              module
                .resource
                .map(|r| r.contains("node_modules"))
                .unwrap_or(false)
            })
            .into(),
          ),
          priority: -10.into(),
          ..Default::default()
        },
      ),
    ]));

    defaults.cache_groups.extend(
      self
        .cache_groups
        .into_iter()
        .map(|(k, v)| {
          (
            k,
            CacheGroupOptions {
              name: v.name.clone().into(),
              priority: 0,
              reuse_existing_chunk: false.into(),
              r#type: SizeType::JavaScript.into(),
              test: Arc::new(move |module| {
                let re = regex::Regex::new(&v.test).unwrap();
                re.is_match(&module.id)
              })
              .into(),
              filename: v.name.into(),
              enforce: false.into(),
              id_hint: Default::default(),
              chunks: ChunkType::All.into(),
              automatic_name_delimiter: "~".to_string().into(),
              max_async_requests: 30,
              max_initial_requests: 30,
              min_chunks: 1,
              min_size: 20000,
              min_size_reduction: 20000,
              enforce_size_threshold: 50000,
              min_remaining_size: 0,
              max_size: 0,
              max_async_size: usize::MAX.into(),
              max_initial_size: usize::MAX.into(),
            },
          )
        })
        .collect(),
    );
    Ok(defaults)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawSplitChunksOptions {
      cache_groups: HashMap::new(),
      chunks: None,
    }
  }
}

#[cfg(feature = "node-api")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCacheGroupOptions {
  //   pub priority: isize,
  //   pub reuse_existing_chunk: bool,
  //   pub r#type: SizeType,
  pub test: String,
  //   pub filename: String,
  //   pub enforce: bool,
  //   pub id_hint: String,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  // TODO: supports function
  pub name: String,
  // used_exports: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawCacheGroupOptions {
  //   pub priority: isize,
  //   pub reuse_existing_chunk: bool,
  //   pub r#type: SizeType,
  pub test: String,
  //   pub filename: String,
  //   pub enforce: bool,
  //   pub id_hint: String,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  // TODO: supports function
  pub name: String,
  // used_exports: bool,
}
