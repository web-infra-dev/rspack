use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{BundleOptions, BundleReactOptions, Loader, ResolveOption};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entries: HashMap<String, String>,
  pub minify: Option<bool>,
  pub root: Option<String>,
  pub outdir: Option<String>,
  pub entry_filename: Option<String>,
  pub loader: Option<HashMap<String, String>>,
  pub inline_style: Option<bool>,
  pub alias: Option<HashMap<String, String>>,
  pub refresh: Option<bool>,
  pub source_map: Option<bool>,
  pub code_splitting: Option<bool>,
  pub svgr: Option<bool>,
  pub lazy_compilation: Option<bool>,
}

pub fn normalize_bundle_options(options: RawOptions) -> BundleOptions {
  let default_options = BundleOptions::default();

  BundleOptions {
    entries: options
      .entries
      .into_iter()
      .map(|(name, src)| (name, src.into()))
      .collect(),
    minify: options.minify.unwrap_or(default_options.minify),
    root: options.root.unwrap_or(default_options.root),
    outdir: options.outdir.unwrap_or(default_options.outdir),
    entry_filename: options
      .entry_filename
      .unwrap_or(default_options.entry_filename),
    loader: options.loader.map_or(Default::default(), parse_loader),
    inline_style: options.inline_style.unwrap_or(default_options.inline_style),
    resolve: ResolveOption {
      alias: options.alias.map_or(default_options.resolve.alias, |op| {
        op.into_iter()
          .map(|(s1, s2)| (s1, Some(s2)))
          .collect::<Vec<_>>()
      }),
      ..Default::default()
    },
    react: BundleReactOptions {
      refresh: options.refresh.unwrap_or(default_options.react.refresh),
      ..Default::default()
    },
    source_map: options.source_map.unwrap_or(default_options.source_map),
    code_splitting: options
      .code_splitting
      .map_or(Some(Default::default()), |flag| {
        if flag {
          Some(Default::default())
        } else {
          None
        }
      }),
    svgr: options.svgr.unwrap_or(default_options.svgr),
    lazy_compilation: options
      .lazy_compilation
      .unwrap_or(BundleOptions::default().lazy_compilation),
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
