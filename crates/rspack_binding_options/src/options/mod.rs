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

impl TryFrom<RawOptions> for CompilerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptions) -> Result<Self, rspack_error::Error> {
    let context: Context = value.context.into();
    let output: OutputOptions = value.output.try_into()?;
    let resolve = value.resolve.try_into()?;
    let resolve_loader = value.resolve_loader.try_into()?;
    let mode = value.mode.unwrap_or_default().into();
    let module: ModuleOptions = value.module.try_into()?;
    let target = Target::new(&value.target)?;
    let cache = value.cache.into();
    let experiments = Experiments {
      incremental_rebuild: IncrementalRebuild {
        make: if matches!(cache, CacheOptions::Disabled) {
          None
        } else {
          Some(IncrementalRebuildMakeState::default())
        },
        emit_asset: true,
      },
      new_split_chunks: value.experiments.new_split_chunks,
      top_level_await: value.experiments.top_level_await,
      rspack_future: value.experiments.rspack_future.into(),
    };
    let optimization: Optimization = IS_ENABLE_NEW_SPLIT_CHUNKS
      .set(&experiments.new_split_chunks, || {
        value.optimization.try_into()
      })?;
    let stats = value.stats.into();
    let snapshot = value.snapshot.into();
    let node = value.node.map(|n| n.into());

    let mut builtins = value.builtins.apply()?;
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
      profile: value.profile,
      bail: value.bail,
      builtins,
    })
  }
}
