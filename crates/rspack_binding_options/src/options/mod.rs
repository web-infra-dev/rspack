use std::{collections::HashMap, fmt::Debug};

use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CompilerOptions, DevServerOptions, Devtool, EntryItem, Experiments, ModuleOptions,
  OutputOptions, PluginExt, TargetPlatform,
};
use serde::Deserialize;

mod raw_builtins;
mod raw_cache;
mod raw_context;
mod raw_dev_server;
mod raw_devtool;
mod raw_entry;
mod raw_experiments;
mod raw_external;
mod raw_mode;
mod raw_module;
mod raw_node;
mod raw_optimization;
mod raw_output;
mod raw_resolve;
mod raw_snapshot;
mod raw_split_chunks;
mod raw_stats;
mod raw_target;

pub use raw_builtins::*;
pub use raw_cache::*;
pub use raw_context::*;
pub use raw_dev_server::*;
pub use raw_devtool::*;
pub use raw_entry::*;
pub use raw_experiments::*;
pub use raw_external::*;
pub use raw_mode::*;
pub use raw_module::*;
pub use raw_node::*;
pub use raw_optimization::*;
pub use raw_output::*;
pub use raw_resolve::*;
pub use raw_snapshot::*;
pub use raw_split_chunks::*;
pub use raw_stats::*;
pub use raw_target::*;

pub trait RawOptionsApply {
  type Options;
  fn apply(self, plugins: &mut Vec<BoxPlugin>) -> Result<Self::Options, rspack_error::Error>;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entry: HashMap<String, RawEntryItem>,
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  #[napi(ts_type = "Array<string>")]
  pub target: RawTarget,
  #[napi(ts_type = "string")]
  pub context: RawContext,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub module: RawModuleOptions,
  pub builtins: RawBuiltins,
  #[napi(ts_type = "Record<string, string>")]
  pub externals: RawExternal,
  pub externals_type: String,
  #[napi(ts_type = "string")]
  pub devtool: RawDevtool,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub dev_server: RawDevServer,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: RawNodeOption,
}

impl RawOptionsApply for RawOptions {
  type Options = CompilerOptions;

  fn apply(self, plugins: &mut Vec<BoxPlugin>) -> Result<Self::Options, rspack_error::Error> {
    let context = self.context.into();
    let entry = self
      .entry
      .into_iter()
      .map(|(name, item)| (name, item.into()))
      .collect::<HashMap<String, EntryItem>>();
    let output: OutputOptions = self.output.apply(plugins)?;
    let resolve = self.resolve.try_into()?;
    let devtool: Devtool = self.devtool.into();
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.try_into()?;
    let target = self.target.apply(plugins)?;
    let externals = vec![self.externals.into()];
    let externals_type = self.externals_type;
    let experiments: Experiments = self.experiments.into();
    let stats = self.stats.into();
    let cache = self.cache.into();
    let snapshot = self.snapshot.into();
    let optimization = self.optimization.apply(plugins)?;
    let node = self.node.into();
    let dev_server: DevServerOptions = self.dev_server.into();
    let builtins = self.builtins.apply(plugins)?;

    plugins.push(
      rspack_plugin_asset::AssetPlugin::new(rspack_plugin_asset::AssetConfig {
        parse_options: module.parser.as_ref().and_then(|x| x.asset.clone()),
      })
      .boxed(),
    );
    plugins.push(rspack_plugin_json::JsonPlugin {}.boxed());
    match &target.platform {
      TargetPlatform::Web => {
        plugins.push(rspack_plugin_runtime::ArrayPushCallbackChunkFormatPlugin {}.boxed());
        plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
        plugins.push(rspack_plugin_runtime::CssModulesPlugin {}.boxed());
        plugins.push(rspack_plugin_runtime::JsonpChunkLoadingPlugin {}.boxed());
      }
      TargetPlatform::Node(_) => {
        plugins.push(rspack_plugin_runtime::CommonJsChunkFormatPlugin {}.boxed());
        plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
        plugins.push(rspack_plugin_runtime::CommonJsChunkLoadingPlugin {}.boxed());
      }
      _ => {
        plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
      }
    };
    if dev_server.hot {
      plugins.push(rspack_plugin_runtime::HotModuleReplacementPlugin {}.boxed());
    }
    plugins.push(rspack_plugin_runtime::BasicRuntimeRequirementPlugin {}.boxed());
    if experiments.lazy_compilation {
      plugins.push(rspack_plugin_runtime::LazyCompilationPlugin {}.boxed());
    }
    plugins.push(rspack_plugin_externals::ExternalPlugin::default().boxed());
    plugins.push(rspack_plugin_javascript::JsPlugin::new().boxed());
    plugins.push(
      rspack_plugin_devtool::DevtoolPlugin::new(rspack_plugin_devtool::DevtoolPluginOptions {
        inline: devtool.inline(),
        append: !devtool.hidden(),
        namespace: output.unique_name.clone(),
        columns: !devtool.cheap(),
        no_sources: devtool.no_sources(),
        public_path: None,
      })
      .boxed(),
    );

    plugins.push(rspack_ids::StableNamedChunkIdsPlugin::new(None, None).boxed());

    // Notice the plugin need to be placed after SplitChunksPlugin
    plugins.push(rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin.boxed());

    Ok(Self::Options {
      entry,
      context,
      mode,
      module,
      target,
      output,
      resolve,
      devtool,
      externals,
      externals_type,
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      dev_server,
      builtins,
    })
  }
}
