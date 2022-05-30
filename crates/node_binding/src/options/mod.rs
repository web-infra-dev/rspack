use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

#[cfg(not(feature = "test"))]
use napi_derive::napi;

use napi::bindgen_prelude::*;
use napi::Error;
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
  pub chunk_filename: Option<String>,
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

pub fn normalize_bundle_options(mut options: RawOptions) -> Result<BundleOptions> {
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
  // let root = options
  //   .root
  //   .unwrap_or_else(|| BundleOptions::default().root);
  let loader = options.loader.unwrap_or_default();
  // let enhanced = options.enhanced.unwrap_or_else(|| mode.into());
  // let optimization = options.optimization.unwrap_or_else(|| mode.into());
  // let output = options.output.unwrap_or_else(|| mode.into());
  // let resolve = options.resolve.unwrap_or_else(|| mode.into());
  // let react = enhanced.react.unwrap_or_else(|| mode.into());
  // let split_chunks = optimization.split_chunks.unwrap_or_else(|| mode.into());

  let source_map = options
    .output
    .as_mut()
    .and_then(|opts| opts.source_map.take())
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

  let defaults: BundleOptions = mode.into();

  Ok(BundleOptions {
    entries: parse_entries(entries),
    root: options.root.unwrap_or(defaults.root),
    mode,
    minify: options
      .optimization
      .as_mut()
      .and_then(|opts| opts.minify.take())
      .unwrap_or(defaults.minify),
    outdir: options
      .output
      .as_mut()
      .and_then(|opts| opts.outdir.take())
      .unwrap_or(defaults.outdir),
    entry_filename: options
      .output
      .and_then(|opts| opts.outdir)
      .unwrap_or(defaults.entry_filename),
    loader: parse_loader(loader),
    inline_style: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.inline_style.take())
      .unwrap_or(defaults.inline_style),
    resolve: ResolveOption {
      alias: options
        .resolve
        .as_mut()
        .and_then(|opts| opts.alias.take())
        .map_or(defaults.resolve.alias, parse_raw_alias),

      condition_names: options
        .resolve
        .as_mut()
        .and_then(|opts| opts.condition_names.take())
        .map_or(defaults.resolve.condition_names, parse_raw_condition_names),
      alias_field: options
        .resolve
        .and_then(|opts| opts.alias_field)
        .unwrap_or(defaults.resolve.alias_field),
      ..Default::default()
    },
    react: BundleReactOptions {
      refresh: options
        .enhanced
        .as_mut()
        .and_then(|opts| opts.react.as_mut())
        .and_then(|opts| opts.fast_refresh.take())
        .unwrap_or(defaults.react.refresh),
      ..Default::default()
    },
    source_map,
    code_splitting: CodeSplittingOptions {
      enable: options
        .optimization
        .as_mut()
        .and_then(|opts| opts.split_chunks.as_mut())
        .and_then(|opts| opts.code_splitting)
        .unwrap_or(defaults.code_splitting.enable),
      reuse_existing_chunk: options
        .optimization
        .as_mut()
        .and_then(|opts| opts.split_chunks.as_mut())
        .and_then(|opts| opts.reuse_exsting_chunk)
        .unwrap_or(defaults.code_splitting.reuse_existing_chunk),
    },
    svgr: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.svgr.take())
      .unwrap_or(defaults.svgr),
    lazy_compilation: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.lazy_compilation.take())
      .unwrap_or(defaults.lazy_compilation),
    progress: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.progress.take())
      .unwrap_or(defaults.progress),
    globals: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.globals.take())
      .unwrap_or(defaults.globals),
    define: options
      .enhanced
      .as_mut()
      .and_then(|opts| opts.define.take())
      .unwrap_or(defaults.define),
    optimization: OptimizationOptions {
      remove_empty_chunks: options
        .optimization
        .as_mut()
        .and_then(|opts| opts.remove_empty_chunks.take())
        .unwrap_or(defaults.optimization.remove_empty_chunks),
      chunk_id_algo: options
        .optimization
        .as_mut()
        .and_then(|opts| opts.chunk_id_algo.take())
        .and_then(|opts| ChunkIdAlgo::from_str(opts.as_str()).ok())
        .unwrap_or(defaults.optimization.chunk_id_algo),
      module_id_algo: options
        .optimization
        .as_mut()
        .and_then(|opts| opts.module_id_algo.take())
        .and_then(|opts| ModuleIdAlgo::from_str(opts.as_str()).ok())
        .unwrap_or(defaults.optimization.module_id_algo),
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

pub fn parse_raw_condition_names(condition_names: Vec<String>) -> HashSet<String> {
  HashSet::from_iter(condition_names.into_iter())
}
