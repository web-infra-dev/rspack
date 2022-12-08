use crate::{
  Builtins, BundleEntries, CacheOptions, Context, DevServerOptions, Devtool, Experiments, External,
  ExternalType, ModuleOptions, OutputOptions, Plugins, Resolve, SnapshotOptions, StatsOptions,
  Target,
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
  pub __emit_error: bool,
  pub module_ids: ModuleIds,
}

#[derive(Debug)]
pub enum ModuleIds {
  Named,
  Deterministic,
}
