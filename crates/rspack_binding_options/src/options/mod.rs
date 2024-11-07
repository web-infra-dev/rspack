use napi_derive::napi;
use rspack_core::{
  incremental::IncrementalPasses, CacheOptions, CompilerOptions, Context, Experiments,
  ModuleOptions, OutputOptions, References,
};

mod raw_builtins;
mod raw_cache;
mod raw_devtool;
mod raw_dynamic_entry;
mod raw_experiments;
mod raw_external;
mod raw_mode;
mod raw_module;
mod raw_node;
mod raw_optimization;
mod raw_output;
mod raw_snapshot;
mod raw_split_chunks;
mod raw_stats;

pub use raw_builtins::*;
pub use raw_cache::*;
pub use raw_devtool::*;
pub use raw_dynamic_entry::*;
pub use raw_experiments::*;
pub use raw_external::*;
pub use raw_mode::*;
pub use raw_module::*;
pub use raw_node::*;
pub use raw_optimization::*;
pub use raw_output::*;
pub use raw_snapshot::*;
pub use raw_split_chunks::*;
pub use raw_stats::*;
pub use rspack_binding_values::raw_resolve::*;

#[derive(Debug)]
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
  #[napi(js_name = "__references", ts_type = "Record<string, any>")]
  pub __references: References,
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
    let cache = value.cache.into();
    let experiments = Experiments {
      incremental: match value.experiments.incremental {
        Some(value) => {
          let mut passes = IncrementalPasses::empty();
          if !matches!(cache, CacheOptions::Disabled) && value.make {
            passes.insert(IncrementalPasses::MAKE);
          }
          if value.infer_async_modules {
            passes.insert(IncrementalPasses::INFER_ASYNC_MODULES);
          }
          if value.provided_exports {
            passes.insert(IncrementalPasses::PROVIDED_EXPORTS);
          }
          if value.dependencies_diagnostics {
            passes.insert(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS);
          }
          if value.build_chunk_graph {
            passes.insert(IncrementalPasses::BUILD_CHUNK_GRAPH);
          }
          if value.modules_hashes {
            passes.insert(IncrementalPasses::MODULES_HASHES);
          }
          if value.modules_codegen {
            passes.insert(IncrementalPasses::MODULES_CODEGEN);
          }
          if value.modules_runtime_requirements {
            passes.insert(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS);
          }
          if value.chunks_runtime_requirements {
            passes.insert(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS);
          }
          if value.emit_assets {
            passes.insert(IncrementalPasses::EMIT_ASSETS);
          }
          passes
        }
        None => IncrementalPasses::empty(),
      },
      layers: value.experiments.layers,
      top_level_await: value.experiments.top_level_await,
      rspack_future: value.experiments.rspack_future.into(),
    };
    let optimization = value.optimization.try_into()?;
    let stats = value.stats.into();
    let snapshot = value.snapshot.into();
    let node = value.node.map(|n| n.into());

    Ok(CompilerOptions {
      context,
      mode,
      module,
      output,
      resolve,
      resolve_loader,
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      profile: value.profile,
      bail: value.bail,
      __references: value.__references,
    })
  }
}
