use std::collections::HashMap;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

use napi::bindgen_prelude::*;

use rspack_core::{
  CompilerOptions, CssOptions, DevServerOptions, EntryItem, OutputAssetModuleFilename,
  OutputOptions, Resolve, Target,
};
// use rspack_core::OptimizationOptions;
// use rspack_core::SourceMapOptions;
// use rspack_core::{
//   BundleMode, BundleOptions, BundleReactOptions, CodeSplittingOptions, EntryItem, Loader,
//   ResolveOption,
// };
// use rspack_core::{ChunkIdAlgo, Platform};
use serde::Deserialize;
use std::path::Path;

// mod enhanced;
// mod optimization;
mod css;
mod output;
// mod react;
// mod resolve;
// mod split_chunks;
// pub use enhanced::*;
// pub use optimization::*;
pub use output::*;

use self::css::RawCssOptions;
// pub use react::*;
// pub use resolve::*;
// pub use split_chunks::*;

#[cfg(not(feature = "test"))]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entries: HashMap<String, String>,
  // #[napi(ts_type = "\"development\" | \"production\" | \"none\"")]
  // pub mode: Option<String>,
  // #[napi(ts_type = "\"browser\" | \"node\"")]
  // pub platform: Option<String>,
  pub root: Option<String>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub css: Option<RawCssOptions>,
  // pub resolve: Option<RawResolveOptions>,
  // pub chunk_filename: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawOptions {
  pub entries: HashMap<String, String>,
  // pub mode: Option<String>,
  // pub platform: Option<String>,
  pub root: Option<String>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub css: Option<RawCssOptions>,
  // pub resolve: Option<RawResolveOptions>,
  // pub chunk_filename: Option<String>,
}

pub fn normalize_bundle_options(mut options: RawOptions) -> Result<CompilerOptions> {
  let cwd = std::env::current_dir().unwrap();

  let root = options
    .root
    .take()
    .unwrap_or_else(|| cwd.to_string_lossy().to_string());

  let output_path = options
    .output
    .as_mut()
    .and_then(|opt| opt.path.take())
    .unwrap_or_else(|| Path::new(&root).join("dist").to_string_lossy().to_string());

  let output_asset_module_filename = options
    .output
    .as_mut()
    .and_then(|opt| opt.asset_module_filename.take())
    .map(OutputAssetModuleFilename::new);

  //Todo the following options is testing, we need inject real options in the user config file
  let public_path = String::from("/");
  let namespace = String::from("__rspack_runtime__");
  let target = Target::String(String::from("web"));
  let resolve = Resolve::default();
  let css_options = {
    let mut css = CssOptions::default();
    css.preset_env = options
      .css
      .as_mut()
      .map(|opt| std::mem::take(&mut opt.preset_env))
      .unwrap_or_default();
    css
  };
  Ok(CompilerOptions {
    entries: parse_entries(options.entries),
    root,
    target,
    dev_server: DevServerOptions { hmr: false },
    output: OutputOptions {
      path: output_path,
      public_path,
      asset_module_filename: output_asset_module_filename.unwrap_or_default(),
      namespace,
    },
    resolve,
    css: css_options,
  })
}

pub fn parse_entries(raw_entry: HashMap<String, String>) -> HashMap<String, EntryItem> {
  raw_entry
    .into_iter()
    .map(|(name, src)| (name, src.into()))
    .collect()
}

// pub fn parse_raw_alias(
//   alias: HashMap<String, ResolveAliasValue>,
// ) -> HashMap<String, Option<String>> {
//   HashMap::from_iter(
//     alias
//       .into_iter()
//       .map(|(key, value)| {
//         let value = match value {
//           ResolveAliasValue::False(b) => {
//             if b {
//               panic!("alias should not be true");
//             } else {
//               None
//             }
//           }
//           ResolveAliasValue::Target(s) => Some(s),
//         };
//         (key, value)
//       })
//       .collect::<Vec<_>>(),
//   )
// }

// pub fn parse_raw_condition_names(condition_names: Vec<String>) -> HashSet<String> {
//   HashSet::from_iter(condition_names.into_iter())
// }
