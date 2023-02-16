use crate::{
  Builtins, BundleEntries, CacheOptions, Context, DevServerOptions, Devtool, Experiments, External,
  ExternalType, ModuleOptions, NodeOption, Optimizations, OutputOptions, Plugins, Resolve,
  SnapshotOptions, StatsOptions, Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: BundleEntries,
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub resolve: Resolve,
  pub builtins: Builtins,
  pub plugins: Plugins,
  pub module: ModuleOptions,
  pub devtool: Devtool,
  pub external: Vec<External>,
  pub external_type: ExternalType,
  pub stats: StatsOptions,
  pub snapshot: SnapshotOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: NodeOption,
  pub __emit_error: bool,
  pub module_ids: ModuleIds,
  pub optimizations: Optimizations,
}

impl CompilerOptions {
  pub fn is_incremental_rebuild(&self) -> bool {
    self.experiments.incremental_rebuild && !matches!(self.cache, CacheOptions::Disabled)
  }
}

#[derive(Debug)]
pub enum ModuleIds {
  Named,
  Deterministic,
}
