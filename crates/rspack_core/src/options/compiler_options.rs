#[cfg(allocative)]
use rspack_util::allocative;

use crate::{
  CacheOptions, Context, Experiments, Mode, ModuleOptions, NodeOption, Optimization, OutputOptions,
  Resolve, SnapshotOptions, StatsOptions, incremental::IncrementalOptions,
};

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct CompilerOptions {
  pub name: Option<String>,
  pub context: Context,
  pub output: OutputOptions,
  pub mode: Mode,
  pub resolve: Resolve,
  pub resolve_loader: Resolve,
  pub module: ModuleOptions,
  pub stats: StatsOptions,
  pub cache: CacheOptions,
  pub snapshot: SnapshotOptions,
  pub experiments: Experiments,
  pub incremental: IncrementalOptions,
  pub node: Option<NodeOption>,
  pub optimization: Optimization,
  pub amd: Option<String>,
  pub bail: bool,
  pub __references: References,
}

pub type References = serde_json::Map<String, serde_json::Value>;
