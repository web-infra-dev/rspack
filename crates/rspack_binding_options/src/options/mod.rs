use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CompilerOptions, DevServerOptions, Devtool, Experiments, IncrementalRebuild,
  IncrementalRebuildMakeState, ModuleOptions, ModuleType, OutputOptions, PluginExt,
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
  fn apply(
    self,
    plugins: &mut Vec<BoxPlugin>,
    loader_runner: &JsLoaderRunner,
  ) -> Result<Self::Options, rspack_error::Error>;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  pub entry: HashMap<String, RawEntryDescription>,
  /// Using this Vector to track the original order of user land entry configuration
  /// std::collection::HashMap does not guarantee the insertion order, for more details you could refer
  /// https://doc.rust-lang.org/std/collections/index.html#iterators:~:text=For%20unordered%20collections%20like%20HashMap%2C%20the%20items%20will%20be%20yielded%20in%20whatever%20order%20the%20internal%20representation%20made%20most%20convenient.%20This%20is%20great%20for%20reading%20through%20all%20the%20contents%20of%20the%20collection.
  pub __entry_order: Vec<String>,
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
  pub externals: Option<Vec<RawExternalItem>>,
  pub externals_type: String,
  pub externals_presets: RawExternalsPresets,
  #[napi(ts_type = "string")]
  pub devtool: RawDevtool,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub dev_server: RawDevServer,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
}

impl RawOptionsApply for RawOptions {
  type Options = CompilerOptions;

  fn apply(
    mut self,
    plugins: &mut Vec<BoxPlugin>,
    loader_runner: &JsLoaderRunner,
  ) -> Result<Self::Options, rspack_error::Error> {
    let context = self.context.into();
    // https://github.com/web-infra-dev/rspack/discussions/3252#discussioncomment-6182939
    // will solve the order problem by add EntryOptionPlugin on js side, and we can only
    // care about EntryOptions instead EntryDescription
    for key in &self.__entry_order {
      if let Some((name, desc)) = self.entry.remove_entry(key) {
        for request in desc.import {
          plugins.push(
            rspack_plugin_entry::EntryPlugin::new(
              name.clone(),
              request,
              rspack_core::EntryOptions {
                runtime: desc.runtime.clone(),
                chunk_loading: desc.chunk_loading.as_deref().map(Into::into),
                public_path: desc.public_path.clone().map(Into::into),
              },
            )
            .boxed(),
          );
        }
      }
    }
    let output: OutputOptions = self.output.apply(plugins, loader_runner)?;
    let resolve = self.resolve.try_into()?;
    let devtool: Devtool = self.devtool.into();
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.apply(plugins, loader_runner)?;
    let target = self.target.apply(plugins, loader_runner)?;
    let cache = self.cache.into();
    let experiments = Experiments {
      lazy_compilation: self.experiments.lazy_compilation,
      incremental_rebuild: IncrementalRebuild {
        make: self
          .experiments
          .incremental_rebuild
          .make
          .then(IncrementalRebuildMakeState::default),
        emit_asset: self.experiments.incremental_rebuild.emit_asset,
      },
      async_web_assembly: self.experiments.async_web_assembly,
      new_split_chunks: self.experiments.new_split_chunks,
      css: self.experiments.css,
    };
    let optimization = IS_ENABLE_NEW_SPLIT_CHUNKS.set(&experiments.new_split_chunks, || {
      self.optimization.apply(plugins, loader_runner)
    })?;
    let stats = self.stats.into();
    let snapshot = self.snapshot.into();
    let node = self.node.map(|n| n.into());
    let dev_server: DevServerOptions = self.dev_server.into();
    let builtins = self.builtins.apply(plugins, loader_runner)?;

    plugins.push(rspack_plugin_schemes::DataUriPlugin.boxed());
    plugins.push(rspack_plugin_schemes::FileUriPlugin.boxed());

    plugins.push(
      rspack_plugin_asset::AssetPlugin::new(rspack_plugin_asset::AssetConfig {
        parse_options: module
          .parser
          .as_ref()
          .and_then(|x| x.get(&ModuleType::Asset))
          .and_then(|x| x.get_asset(&ModuleType::Asset).cloned()),
      })
      .boxed(),
    );
    plugins.push(rspack_plugin_json::JsonPlugin {}.boxed());
    if dev_server.hot {
      plugins.push(rspack_plugin_runtime::HotModuleReplacementPlugin {}.boxed());
    }
    plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
    if experiments.lazy_compilation {
      plugins.push(rspack_plugin_runtime::LazyCompilationPlugin {}.boxed());
    }
    if let Some(externals) = self.externals {
      plugins.push(
        rspack_plugin_externals::ExternalPlugin::new(
          self.externals_type,
          externals
            .into_iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<_>, _>>()?,
        )
        .boxed(),
      );
    }
    if self.externals_presets.node {
      plugins.push(rspack_plugin_externals::node_target_plugin());
    }
    if self.externals_presets.electron_main {
      rspack_plugin_externals::electron_target_plugin(
        rspack_plugin_externals::ElectronTargetContext::Main,
        plugins,
      );
    }
    if self.externals_presets.electron_preload {
      rspack_plugin_externals::electron_target_plugin(
        rspack_plugin_externals::ElectronTargetContext::Preload,
        plugins,
      );
    }
    if self.externals_presets.electron_renderer {
      rspack_plugin_externals::electron_target_plugin(
        rspack_plugin_externals::ElectronTargetContext::Renderer,
        plugins,
      );
    }
    if self.externals_presets.electron
      && !self.externals_presets.electron_main
      && !self.externals_presets.electron_preload
      && !self.externals_presets.electron_renderer
    {
      rspack_plugin_externals::electron_target_plugin(
        rspack_plugin_externals::ElectronTargetContext::None,
        plugins,
      );
    }
    if self.externals_presets.web || (self.externals_presets.node && experiments.css) {
      plugins.push(rspack_plugin_externals::http_url_external_plugin(
        experiments.css,
      ));
    }
    if experiments.async_web_assembly {
      plugins.push(rspack_plugin_wasm::AsyncWasmPlugin::new().boxed());
    }
    plugins.push(rspack_plugin_javascript::JsPlugin::new().boxed());
    plugins.push(rspack_plugin_javascript::InferAsyncModulesPlugin {}.boxed());
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
    if optimization.remove_empty_chunks {
      plugins.push(rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin.boxed());
    }

    Ok(Self::Options {
      context,
      mode,
      module,
      target,
      output,
      resolve,
      devtool,
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
