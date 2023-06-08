use crate::{
  Builtins, BundleEntries, CacheOptions, Context, DevServerOptions, Devtool, Experiments, Mode,
  ModuleOptions, NodeOption, Optimization, OutputOptions, Resolve, SnapshotOptions, StatsOptions,
  Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: BundleEntries,
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  // TODO(swc-loader): target should not exist on compiler options
  pub target: Target,
  pub mode: Mode,
  pub resolve: Resolve,
  pub builtins: Builtins,
  pub module: ModuleOptions,
  pub devtool: Devtool,
  pub stats: StatsOptions,
  pub snapshot: SnapshotOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: Option<NodeOption>,
  pub optimization: Optimization,
}

impl CompilerOptions {
  pub fn is_make_use_incremental_rebuild(&self) -> bool {
    self.experiments.incremental_rebuild.make && !matches!(self.cache, CacheOptions::Disabled)
  }
}
