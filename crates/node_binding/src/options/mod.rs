use std::{collections::HashMap, str::FromStr};

use napi_derive::napi;
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

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub mode: Option<String>,
  pub entries: HashMap<String, String>,
  pub root: Option<String>,
  pub loader: Option<HashMap<String, String>>,
  pub enhanced: Option<RawEnhancedOptions>,
  pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
}

pub fn normalize_bundle_options(options: RawOptions) -> BundleOptions {
  let mode: BundleMode = options.mode.map_or_else(
    || {
      // We might need to read NODE_ENV to try to resolve a BundleMode.
      BundleMode::None
    },
    |mode_str| BundleMode::from_str(mode_str.as_str()).expect("unexpected bundle mode"),
  );

  let entries = options.entries;
  let root = options
    .root
    .unwrap_or_else(|| BundleOptions::default().root);
  let loader = options.loader.unwrap_or_default();
  let enhanced = options.enhanced.unwrap_or_else(|| mode.into());
  let optimization = options.optimization.unwrap_or_else(|| mode.into());
  let outout = options.output.unwrap_or_else(|| mode.into());
  let resolve = options.resolve.unwrap_or_else(|| mode.into());
  let react = enhanced.react.unwrap_or_else(|| mode.into());
  let split_chunks = optimization.split_chunks.unwrap_or_else(|| mode.into());
  BundleOptions {
    entries: parse_entries(entries),
    root,
    minify: optimization.minify.unwrap_or_else(|| mode.is_prod()),
    outdir: outout
      .outdir
      .unwrap_or_else(|| BundleOptions::default().outdir),
    entry_filename: outout
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
    source_map: outout.source_map.unwrap_or(true),
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
    ..Default::default()
  }
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
