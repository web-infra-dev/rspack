use crate::{
  Builtins, CacheOptions, Context, DevServerOptions, Devtool, Experiments,
  IncrementalRebuildMakeState, Mode, ModuleOptions, NodeOption, Optimization, OutputOptions,
  Resolve, SnapshotOptions, StatsOptions, Target,
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
  pub devtool: Devtool,
  pub stats: StatsOptions,
  pub snapshot: SnapshotOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: Option<NodeOption>,
  pub optimization: Optimization,
  pub profile: bool,
  pub builtins: Builtins,
}

impl CompilerOptions {
  pub fn is_incremental_rebuild_make_enabled(&self) -> bool {
    self.experiments.incremental_rebuild.make.is_some()
  }

  pub fn get_incremental_rebuild_make_state(&self) -> Option<&IncrementalRebuildMakeState> {
    self.experiments.incremental_rebuild.make.as_ref()
  }

  pub fn is_incremental_rebuild_emit_asset_enabled(&self) -> bool {
    self.experiments.incremental_rebuild.emit_asset
  }

  pub fn is_new_tree_shaking(&self) -> bool {
    self.experiments.rspack_future.new_treeshaking
  }

  /// This controls whether we're using the old Rspack behavior to transform a module.
  /// - `true`: use the old strategy, and transform the non web standard modules into a standardized one.
  ///           `Rule.type` that are not supported by webpack is not usable.
  /// - `false`: use the new strategy, only web-standard modules are supported.
  pub fn should_transform_by_default(&self) -> bool {
    !self.experiments.rspack_future.disable_transform_by_default
  }
}
