use std::sync::Arc;

use rspack_loader_runner::LoaderMetadata;
use rustc_hash::FxHashMap;

use crate::{
  CacheOptions, Context, Experiments, Mode, ModuleOptions, NodeOption, Optimization, OutputOptions,
  Resolve, SnapshotOptions, StatsOptions, incremental::IncrementalOptions,
};

pub type LoaderMetadataMap = FxHashMap<String, Arc<LoaderMetadata>>;

#[derive(Debug)]
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
  pub __loader_metadata: LoaderMetadataMap,
}

pub type References = serde_json::Map<String, serde_json::Value>;
