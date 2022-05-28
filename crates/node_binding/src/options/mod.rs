use std::{collections::HashMap, str::FromStr};

use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;
use rspack_core::ChunkIdAlgo;
use rspack_core::ModuleIdAlgo;
use rspack_core::OptimizationOptions;
use rspack_core::SourceMapOptions;
use rspack_core::{
  BundleMode, BundleOptions, BundleReactOptions, CodeSplittingOptions, EntryItem, Loader,
  ResolveOption,
};
use serde::Deserialize;

mod enhanced;
mod optimization;
mod output;
mod react;
mod resolve;
mod split_chunks;
pub use enhanced::*;
pub use optimization::*;
pub use output::*;
pub use react::*;
pub use resolve::*;
pub use split_chunks::*;

#[cfg(not(feature = "test"))]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entries: HashMap<String, String>,
  #[napi(ts_type = "\"development\" | \"production\" | \"none\"")]
  pub mode: Option<String>,
  pub root: Option<String>,
  pub loader: Option<HashMap<String, String>>,
  pub enhanced: Option<RawEnhancedOptions>,
  pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawOptions {
  pub entries: HashMap<String, String>,
  pub mode: Option<String>,
  pub root: Option<String>,
  pub loader: Option<HashMap<String, String>>,
  pub enhanced: Option<RawEnhancedOptions>,
  pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
  pub chunk_filename: Option<String>,
}

pub fn normalize_bundle_options(options: RawOptions) -> Result<BundleOptions> {
  let mut err = None;

  let mode = options
    .mode
    .map(|mode| match mode.as_str() {
      "development" => BundleMode::Dev,
      "production" => BundleMode::Prod,
      "none" => BundleMode::None,
      _ => {
        err = Some(Error::new(
          napi::Status::InvalidArg,
          "invalid mode, expected mode 'development' | 'production' | 'none'".to_owned(),
        ));
        BundleMode::None
      }
    })
    .unwrap_or_else(|| BundleOptions::default().mode);

  let entries = options.entries;
  let root = options
    .root
    .unwrap_or_else(|| BundleOptions::default().root);
  let loader = options.loader.unwrap_or_default();
  let enhanced = options.enhanced.unwrap_or_else(|| mode.into());
  let optimization = options.optimization.unwrap_or_else(|| mode.into());
  let output = options.output.unwrap_or_else(|| mode.into());
  let resolve = options.resolve.unwrap_or_else(|| mode.into());
  let react = enhanced.react.unwrap_or_else(|| mode.into());
  let split_chunks = optimization.split_chunks.unwrap_or_else(|| mode.into());

  let source_map = output
    .source_map
    .map(|source_map| match source_map.as_str() {
      "none" => SourceMapOptions::None,
      "inline" => SourceMapOptions::Inline,
      "external" => SourceMapOptions::External,
      "linked" => SourceMapOptions::Linked,
      _ => {
        err = Some(Error::new(
          napi::Status::InvalidArg,
          "invalid option `source_map`, expected options 'inline' | 'external' | 'linked' | 'none'"
            .to_owned(),
        ));
        SourceMapOptions::None
      }
    })
    .unwrap_or_else(|| true.into());

  if let Some(e) = err {
    return Err(e);
  }

  Ok(BundleOptions {
    entries: parse_entries(entries),
    root,
    mode,
    minify: optimization.minify.unwrap_or_else(|| mode.is_prod()),
    outdir: output
      .outdir
      .unwrap_or_else(|| BundleOptions::default().outdir),
    entry_filename: output
      .entry_filename
      .unwrap_or_else(|| BundleOptions::default().entry_filename),
    loader: parse_loader(loader),
    inline_style: enhanced.inline_style.unwrap_or(false),
    resolve: ResolveOption {
      alias: resolve.alias.map_or(Default::default(), parse_raw_alias),
      ..Default::default()
    },
    react: BundleReactOptions {
      refresh: react.fast_fresh.unwrap_or_else(|| mode.is_dev()),
      ..Default::default()
    },
    source_map,
    code_splitting: split_chunks
      .code_splitting
      .map(|is_enable| {
        if is_enable {
          Some(CodeSplittingOptions {
            reuse_existing_chunk: split_chunks
              .reuse_exsting_chunk
              .unwrap_or_else(|| !mode.is_none()),
          })
        } else {
          None
        }
      })
      .unwrap_or_default(),
    svgr: enhanced.svgr.unwrap_or(false),
    lazy_compilation: enhanced.lazy_compilation.unwrap_or(false),
    progress: enhanced.progress.unwrap_or(true),
    globals: enhanced.globals.unwrap_or_default(),
    define: enhanced
      .define
      .unwrap_or_else(|| BundleOptions::default().define),
    optimization: OptimizationOptions {
      remove_empty_chunks: optimization.remove_empty_chunks.unwrap_or(!mode.is_none()),
      chunk_id_algo: optimization
        .chunk_id_algo
        .map_or(ChunkIdAlgo::Named, |algo_str| {
          ChunkIdAlgo::from_str(algo_str.as_str()).unwrap()
        }),
      module_id_algo: optimization
        .module_id_algo
        .map_or(ModuleIdAlgo::Named, |algo_str| {
          ModuleIdAlgo::from_str(algo_str.as_str()).unwrap()
        }),
    },
    chunk_filename: options
      .chunk_filename
      .unwrap_or_else(|| BundleOptions::default().chunk_filename),
    ..Default::default()
  })
}

fn parse_loader(user_input: HashMap<String, String>) -> rspack_core::LoaderOptions {
  let loaders = Loader::values()
    .into_iter()
    .map(|loader| match loader {
      Loader::Css => ("css", loader),
      Loader::Less => ("less", loader),
      Loader::Sass => ("sass", loader),
      Loader::DataURI => ("dataURI", loader),
      Loader::Js => ("js", loader),
      Loader::Jsx => ("jsx", loader),
      Loader::Ts => ("ts", loader),
      Loader::Tsx => ("tsx", loader),
      Loader::Null => ("null", loader),
      Loader::Json => ("json", loader),
      Loader::Text => ("text", loader),
    })
    .collect::<HashMap<_, _>>();
  user_input
    .into_iter()
    .filter_map(|(ext, loader_str)| {
      let loader = *loaders.get(loader_str.as_str())?;
      Some((ext, loader))
    })
    .collect()
}

pub fn parse_entries(raw_entry: HashMap<String, String>) -> HashMap<String, EntryItem> {
  raw_entry
    .into_iter()
    .map(|(name, src)| (name, src.into()))
    .collect()
}

pub fn parse_raw_alias(alias: HashMap<String, String>) -> Vec<(String, Option<String>)> {
  alias
    .into_iter()
    .map(|(s1, s2)| (s1, Some(s2)))
    .collect::<Vec<_>>()
}
