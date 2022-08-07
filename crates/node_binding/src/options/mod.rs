use std::collections::HashMap;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

use napi::bindgen_prelude::*;

use rspack_core::{
  CompilerOptions, DevServerOptions, EntryItem, OutputAssetModuleFilename, OutputOptions, Resolve,
  Target,
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
mod output;
// mod react;
// mod resolve;
// mod split_chunks;
// pub use enhanced::*;
// pub use optimization::*;
pub use output::*;
// pub use react::*;
// pub use resolve::*;
// pub use split_chunks::*;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "test"), napi(object))]
pub struct RawOptions {
  pub entry: HashMap<String, String>,
  // #[napi(ts_type = "\"development\" | \"production\" | \"none\"")]
  // pub mode: Option<String>,
  // #[napi(ts_type = "\"browser\" | \"node\"")]
  // pub platform: Option<String>,
  pub context: Option<String>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  // pub resolve: Option<RawResolveOptions>,
  // pub chunk_filename: Option<String>,
}

pub fn normalize_bundle_options(mut options: RawOptions) -> Result<CompilerOptions> {
  let cwd = std::env::current_dir().unwrap();

  let context = options
    .context
    .take()
    .unwrap_or_else(|| cwd.to_string_lossy().to_string());

  let output_path = options
    .output
    .as_mut()
    .and_then(|opt| opt.path.take())
    .unwrap_or_else(|| {
      Path::new(&context)
        .join("dist")
        .to_string_lossy()
        .to_string()
    });

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
  Ok(CompilerOptions {
    entry: parse_entries(options.entry),
    context,
    target,
    dev_server: DevServerOptions { hmr: false },
    output: OutputOptions {
      path: output_path,
      public_path,
      asset_module_filename: output_asset_module_filename.unwrap_or_default(),
      namespace,
    },
    resolve,
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
