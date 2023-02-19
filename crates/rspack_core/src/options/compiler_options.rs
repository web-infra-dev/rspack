use crate::{
  Builtins, BundleEntries, CacheOptions, Context, DevServerOptions, Devtool, Experiments,
  ExternalType, Externals, Mode, ModuleOptions, NodeOption, Optimization, OutputOptions, Resolve,
  SnapshotOptions, StatsOptions, Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: BundleEntries,
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub mode: Mode,
  pub resolve: Resolve,
  pub builtins: Builtins,
  pub module: ModuleOptions,
  pub devtool: Devtool,
  pub externals: Externals,
  pub externals_type: ExternalType,
  pub stats: StatsOptions,
  pub snapshot: SnapshotOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: NodeOption,
  pub optimization: Optimization,
}

impl CompilerOptions {
  pub fn is_incremental_rebuild(&self) -> bool {
    self.experiments.incremental_rebuild && !matches!(self.cache, CacheOptions::Disabled)
  }
}
