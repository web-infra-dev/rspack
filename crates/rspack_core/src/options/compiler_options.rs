use super::Incremental;
use crate::{
  CacheOptions, Context, DevServerOptions, Experiments, Mode, ModuleOptions, NodeOption,
  Optimization, OutputOptions, Resolve, SnapshotOptions, StatsOptions, Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  // TODO(swc-loader): target should not exist on compiler options
  pub target: Target,
  pub mode: Mode,
  pub resolve: Resolve,
  pub resolve_loader: Resolve,
  pub module: ModuleOptions,
  pub stats: StatsOptions,
  pub snapshot: SnapshotOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: Option<NodeOption>,
  pub optimization: Optimization,
  pub profile: bool,
  pub bail: bool,
  pub __references: References,
}

pub type References = serde_json::Map<String, serde_json::Value>;

impl CompilerOptions {
  pub fn incremental(&self) -> &Incremental {
    &self.experiments.incremental
  }
}
