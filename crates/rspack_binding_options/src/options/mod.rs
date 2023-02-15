use std::{collections::HashMap, fmt::Debug};

use napi_derive::napi;
use rspack_core::{CompilerOptions, CompilerOptionsBuilder, EntryItem, ModuleIds};
use serde::Deserialize;

mod raw_builtins;
mod raw_cache;
mod raw_context;
mod raw_dev_server;
mod raw_devtool;
mod raw_entry;
mod raw_experiments;
mod raw_external;
mod raw_external_type;
mod raw_mode;
mod raw_module;
mod raw_node;
mod raw_optimization;
mod raw_output;
mod raw_plugins;
mod raw_resolve;
mod raw_snapshot;
mod raw_split_chunks;
mod raw_stats;
mod raw_target;

pub use raw_builtins::*;
pub use raw_cache::*;
pub use raw_context::*;
pub use raw_dev_server::*;
pub use raw_entry::*;
pub use raw_experiments::*;
pub use raw_external::*;
pub use raw_external_type::*;
pub use raw_mode::*;
pub use raw_module::*;
pub use raw_node::*;
pub use raw_optimization::*;
pub use raw_output::*;
pub use raw_plugins::*;
pub use raw_resolve::*;
pub use raw_snapshot::*;
pub use raw_split_chunks::*;
pub use raw_stats::*;
pub use raw_target::*;

use self::raw_devtool::RawDevtool;
pub trait RawOption<T> {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<T>;
  /// use to create default value when input is `None`.
  fn fallback_value(options: &CompilerOptionsBuilder) -> Self;
  fn raw_to_compiler_option(
    raw: Option<Self>,
    options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<T>
  where
    Self: Sized + Debug,
  {
    match raw {
      Some(value) => value,
      None => Self::fallback_value(options),
    }
    .to_compiler_option(options)
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entry: Option<HashMap<String, RawEntryItem>>,
  #[napi(ts_type = "string")]
  pub mode: Option<RawMode>,
  #[napi(ts_type = "string[]")]
  pub target: Option<RawTarget>,
  // #[napi(ts_type = "\"browser\" | \"node\"")]
  // pub platform: Option<String>,
  #[napi(ts_type = "string")]
  pub context: Option<RawContext>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
  pub module: Option<RawModuleOptions>,
  pub builtins: Option<RawBuiltins>,
  #[napi(ts_type = "Record<string, string>")]
  pub externals: Option<RawExternal>,
  #[napi(ts_type = "string")]
  pub externals_type: Option<RawExternalType>,
  #[napi(ts_type = "string")]
  pub devtool: Option<RawDevtool>,
  pub optimization: Option<RawOptimizationOptions>,
  pub stats: Option<RawStatsOptions>,
  pub dev_server: Option<RawDevServer>,
  pub snapshot: Option<RawSnapshotOptions>,
  pub cache: Option<RawCacheOptions>,
  pub experiments: Option<RawExperiments>,
  pub node: Option<RawNodeOption>,
}

pub fn normalize_bundle_options(raw_options: RawOptions) -> anyhow::Result<CompilerOptions> {
  // normalize_options should ensuring orderliness.
  let compiler_options = CompilerOptionsBuilder::default()
    .then(|mut options| {
      let context = RawOption::raw_to_compiler_option(raw_options.context, &options)?;
      options.context = Some(context);
      Ok(options)
    })?
    .then(|mut options| {
      let mode = RawOption::raw_to_compiler_option(raw_options.mode, &options)?;
      let module_ids = raw_options
        .optimization
        .as_ref()
        .and_then(|o| o.module_ids.as_ref())
        .map(|value| match value.as_str() {
          "named" => ModuleIds::Named,
          "deterministic" => ModuleIds::Deterministic,
          _ => unimplemented!(),
        })
        .unwrap_or_else(|| match mode {
          rspack_core::Mode::Development => ModuleIds::Named,
          rspack_core::Mode::Production => ModuleIds::Deterministic,
        });
      options.mode = Some(mode);
      options.module_ids = Some(module_ids);
      Ok(options)
    })?
    .then(|mut options| {
      let entry = raw_options
        .entry
        .map(|item| {
          item
            .into_iter()
            .map(|(key, value)| {
              (
                key,
                EntryItem {
                  runtime: value.runtime,
                  import: value.import,
                },
              )
            })
            .collect::<HashMap<String, EntryItem>>()
        })
        .unwrap_or_default();
      options.entry = Some(entry);
      Ok(options)
    })?
    .then(|mut options| {
      let output = RawOption::raw_to_compiler_option(raw_options.output, &options)?;
      options.output = Some(output);
      Ok(options)
    })?
    .then(|mut options| {
      let target = RawOption::raw_to_compiler_option(raw_options.target, &options)?;
      options.target = Some(target);
      Ok(options)
    })?
    .then(|mut options| {
      let resolve = RawOption::raw_to_compiler_option(raw_options.resolve, &options)?;
      options.resolve = Some(resolve);
      Ok(options)
    })?
    .then(|mut options| {
      let mut plugins = vec![];
      let builtins = raw_options.builtins.unwrap_or_default();
      let res_builtins = normalize_builtin(builtins, &mut plugins, &options)?;
      options.plugins = Some(plugins);
      options.builtins = Some(res_builtins);
      Ok(options)
    })?
    .then(|mut options| {
      // TODO: remove or keep.
      let dev_server = RawOption::raw_to_compiler_option(raw_options.dev_server, &options)?;
      options.dev_server = Some(dev_server);
      Ok(options)
    })?
    .then(|mut options| {
      let module_options = RawOption::raw_to_compiler_option(raw_options.module, &options)?;
      options.module = module_options;
      Ok(options)
    })?
    .then(|mut options| {
      let devtool = RawOption::raw_to_compiler_option(raw_options.devtool, &options)?;
      options.devtool = Some(devtool);
      Ok(options)
    })?
    .then(|mut options| {
      let external = RawOption::raw_to_compiler_option(raw_options.externals, &options)?;
      options.external = Some(external);
      Ok(options)
    })?
    .then(|mut options| {
      let external_type = RawOption::raw_to_compiler_option(raw_options.externals_type, &options)?;
      options.external_type = Some(external_type);
      Ok(options)
    })?
    .then(|mut options| {
      let optimization = options.optimizations.get_or_insert_with(|| {
        let is_prod = options
          .mode
          .map_or(false, |mode| mode == rspack_core::Mode::Production);
        rspack_core::Optimizations {
          remove_available_modules: is_prod,
          side_effects: if is_prod {
            "true".into()
          } else {
            "false".into()
          },
        }
      });
      if let Some(raw_optimization) = raw_options.optimization.as_ref() {
        optimization.remove_available_modules = raw_optimization
          .remove_available_modules
          .unwrap_or(optimization.remove_available_modules);
        optimization.side_effects = raw_optimization
          .side_effects
          .as_ref()
          .map(|side_effects| side_effects.as_str().into())
          .unwrap_or(optimization.side_effects);
      }
      Ok(options)
    })?
    .then(|mut options| {
      // if dev_friendly_split_chunks is enabled, we will ignore the split_chunks option.
      if options
        .builtins
        .as_ref()
        .map(|opts| opts.dev_friendly_split_chunks)
        .unwrap_or_default()
      {
        options.plugins.get_or_insert_default().push(Box::new(
          rspack_plugin_dev_friendly_split_chunks::DevFriendlySplitChunksPlugin::new(),
        ));
      } else if let Some(optimization) = raw_options.optimization {
        let split_chunks = RawOption::raw_to_compiler_option(optimization.split_chunks, &options)?;
        options.plugins.get_or_insert_default().push(Box::new(
          rspack_plugin_split_chunks::SplitChunksPlugin::new(split_chunks),
        ))
      }
      Ok(options)
    })?
    .then(|mut options| {
      let stats = RawOption::raw_to_compiler_option(raw_options.stats, &options)?;
      options.stats = Some(stats);
      Ok(options)
    })?
    .then(|mut options| {
      let snapshot = RawOption::raw_to_compiler_option(raw_options.snapshot, &options)?;
      options.snapshot = Some(snapshot);
      Ok(options)
    })?
    .then(|mut options| {
      let cache = RawOption::raw_to_compiler_option(raw_options.cache, &options)?;
      options.cache = Some(cache);
      Ok(options)
    })?
    .then(|mut options| {
      let experiments = RawOption::raw_to_compiler_option(raw_options.experiments, &options)?;
      options.experiments = Some(experiments);
      Ok(options)
    })?
    .then(|mut options| {
      let node = RawOption::raw_to_compiler_option(raw_options.node, &options)?;
      options.node = Some(node);
      Ok(options)
    })?
    .finish();

  Ok(compiler_options)
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

#[cfg(test)]
mod test {
  use crate::normalize_bundle_options;

  #[test]
  fn empty_test() {
    let raw = serde_json::from_str("{}").expect("TODO:");
    let options = normalize_bundle_options(raw).expect("TODO:");
    assert!(&options
      .output
      .path
      .to_string_lossy()
      .contains("rspack_binding_options/dist"));
  }
}
