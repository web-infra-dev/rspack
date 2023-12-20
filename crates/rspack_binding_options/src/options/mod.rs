use napi_derive::napi;
use rspack_core::{
  CompilerOptions, Context, DevServerOptions, Devtool, Experiments, IncrementalRebuild,
  IncrementalRebuildMakeState, MangleExportsOption, ModuleOptions, ModuleType, Optimization,
  OutputOptions, PluginExt, Target, TreeShaking,
};
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, MangleExportsPlugin,
  SideEffectsFlagPlugin,
};
use serde::Deserialize;

mod raw_builtins;
mod raw_cache;
mod raw_dev_server;
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

pub use raw_builtins::*;
pub use raw_cache::*;
pub use raw_dev_server::*;
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

pub trait RawOptionsApply {
  type Options;
  fn apply(self) -> Result<Self::Options, rspack_error::Error>;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptions {
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  pub target: Vec<String>,
  pub context: String,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub resolve_loader: RawResolveOptions,
  pub module: RawModuleOptions,
  pub devtool: String,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub dev_server: RawDevServer,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub builtins: RawBuiltins,
}

impl RawOptions {
  pub fn apply(
    self,
    plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> rspack_error::Result<CompilerOptions> {
    let context: Context = self.context.into();
    let output: OutputOptions = self.output.try_into()?;
    let resolve = self.resolve.try_into()?;
    let resolve_loader = self.resolve_loader.try_into()?;
    let devtool: Devtool = self.devtool.into();
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.try_into()?;
    let target = Target::new(&self.target)?;
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
      new_split_chunks: self.experiments.new_split_chunks,
      top_level_await: self.experiments.top_level_await,
      rspack_future: self.experiments.rspack_future.into(),
    };
    let optimization: Optimization = IS_ENABLE_NEW_SPLIT_CHUNKS
      .set(&experiments.new_split_chunks, || {
        self.optimization.try_into()
      })?;
    let stats = self.stats.into();
    let snapshot = self.snapshot.into();
    let node = self.node.map(|n| n.into());
    let dev_server: DevServerOptions = self.dev_server.into();

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

    if devtool.source_map() {
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
    }

    if experiments.rspack_future.new_treeshaking {
      if optimization.side_effects.is_enable() {
        plugins.push(SideEffectsFlagPlugin::default().boxed());
      }
      if optimization.provided_exports {
        plugins.push(FlagDependencyExportsPlugin::default().boxed());
      }
      if optimization.used_exports.is_enable() {
        plugins.push(FlagDependencyUsagePlugin::default().boxed());
      }
    }
    if optimization.mangle_exports.is_enable() {
      // We already know mangle_exports != false
      plugins.push(
        MangleExportsPlugin::new(!matches!(
          optimization.mangle_exports,
          MangleExportsOption::Size
        ))
        .boxed(),
      );
    }

    let mut builtins = self.builtins.apply(plugins)?;
    if experiments.rspack_future.new_treeshaking {
      builtins.tree_shaking = TreeShaking::False;
    }

    Ok(CompilerOptions {
      context,
      mode,
      module,
      target,
      output,
      resolve,
      resolve_loader,
      devtool,
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      dev_server,
      profile: self.profile,
      builtins,
    })
  }
}
