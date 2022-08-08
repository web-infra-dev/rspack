use std::collections::HashMap;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

use napi::bindgen_prelude::*;

use rspack_core::{
  CompilerOptions, DevServerOptions, EntryItem, OutputAssetModuleFilename, OutputOptions, Plugin,
  Resolve, Target,
};
use rspack_plugin_html::config::HtmlPluginConfig;
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

pub type RawPluginOptions = serde_json::value::Value;
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "test"))]
#[napi(object)]
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
  #[napi(ts_type = "any[]")]
  pub plugins: Option<RawPluginOptions>,
}

// This is a clone of structure above, and for feature=test only
// the reason remains unclear
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawOptions {
  pub entry: HashMap<String, String>,
  pub context: Option<String>,
  pub output: Option<RawOutputOptions>,
  pub plugins: Option<RawPluginOptions>,
}

pub fn create_plugins(options: &RawOptions) -> Result<Vec<Box<dyn Plugin>>> {
  let mut result: Vec<Box<dyn Plugin>> = vec![];
  let plugins = options.plugins.as_ref();
  if plugins.is_none() {
    return Ok(result);
  }
  if let Some(plugins) = plugins.unwrap().as_array() {
    for (i, plugin) in plugins.iter().enumerate() {
      let (target, config) = if let Some(name) = plugin.as_str() {
        (Some(name.to_ascii_lowercase()), None)
      } else if let Some(name_with_config) = plugin.as_array() {
        (
          name_with_config
            .get(0)
            .and_then(|f| f.as_str())
            .map(|f| f.to_ascii_lowercase()),
          name_with_config.get(1),
        )
      } else {
        return Err(napi::Error::from_reason(format!(
          "`config.plugins[{i}]`: structure is not recognized."
        )));
      };

      match target.as_deref() {
        Some("html") => {
          let config: HtmlPluginConfig = match config {
            Some(config) => serde_json::from_value::<HtmlPluginConfig>(config.clone())?,
            None => Default::default(),
          };
          result.push(Box::new(rspack_plugin_html::HtmlPlugin::new(config)));
        }
        _ => {
          return Err(napi::Error::from_reason(format!(
            "`config.plugins[{i}]`: plugin is not found."
          )));
        }
      };
    }
  } else {
    return Err(napi::Error::from_reason(format!(
      "`config.plugins`: structure is not recognized. Found `{:?}`",
      plugins
    )));
  }
  Ok(result)
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
  let plugins = create_plugins(&options)?;
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
    plugins,
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
