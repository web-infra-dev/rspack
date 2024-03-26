use napi_derive::napi;
use rspack_core::{
  CacheOptions, CompilerOptions, Context, Experiments, IncrementalRebuild,
  IncrementalRebuildMakeState, ModuleOptions, Optimization, OutputOptions, Target, TreeShaking,
};
use serde::Deserialize;

mod raw_builtins;
mod raw_cache;
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

pub use raw_builtins::*;
pub use raw_cache::*;
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

pub trait RawOptionsApply {
  type Options;
  fn apply(self) -> Result<Self::Options, rspack_error::Error>;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
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
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub bail: bool,
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
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.try_into()?;
    let target = Target::new(&self.target)?;
    let cache = self.cache.into();
    let experiments = Experiments {
      incremental_rebuild: IncrementalRebuild {
        make: if matches!(cache, CacheOptions::Disabled) {
          None
        } else {
          Some(IncrementalRebuildMakeState::default())
        },
        emit_asset: true,
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
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      dev_server: Default::default(),
      profile: self.profile,
      bail: self.bail,
      builtins,
    })
  }
}
