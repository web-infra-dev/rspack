mod compiler_options;

pub use compiler_options::*;
mod devtool;
pub use devtool::*;
mod entry;
pub use entry::*;
mod optimizations;
pub use optimizations::*;
mod dev_server;
pub use dev_server::*;
mod output;
pub use output::*;
mod target;
pub use target::*;
mod resolve;
pub use resolve::*;
mod mode;
pub use mode::*;
mod builtins;
pub use builtins::*;
mod context;
pub use context::*;
mod plugins;
pub use plugins::*;
mod module;
pub use module::*;
mod external;
pub use external::*;
mod stats;
pub use stats::*;
mod cache;
pub use cache::*;
mod snapshot;
pub use snapshot::*;
mod experiments;
pub use experiments::*;
mod node;
pub use node::*;

#[derive(Debug, Default)]
pub struct CompilerOptionsBuilder {
  pub entry: Option<BundleEntries>,
  pub mode: Option<Mode>,
  pub context: Option<Context>,
  pub dev_server: Option<DevServerOptions>,
  pub output: Option<OutputOptions>,
  pub target: Option<Target>,
  pub resolve: Option<Resolve>,
  pub builtins: Option<Builtins>,
  pub plugins: Option<Plugins>,
  pub module: Option<ModuleOptions>,
  pub devtool: Option<Devtool>,
  pub external: Option<Vec<External>>,
  pub external_type: Option<ExternalType>,
  pub stats: Option<StatsOptions>,
  pub snapshot: Option<SnapshotOptions>,
  pub cache: Option<CacheOptions>,
  pub module_ids: Option<ModuleIds>,
  pub experiments: Option<Experiments>,
  pub node: Option<NodeOption>,
  pub optimizations: Option<Optimizations>,
}

impl CompilerOptionsBuilder {
  /// ## Warning
  /// Caller should ensure that all fields of [CompilerOptionsBuilder] are not `None`.
  /// Otherwise, this function will panic during the runtime
  pub fn finish(self) -> CompilerOptions {
    CompilerOptions {
      entry: self.entry.expect("build options.entry failed"),
      context: self.context.expect("build options.context failed"),
      dev_server: self.dev_server.expect("build options.dev_server failed"),
      output: self.output.expect("build options.output failed"),
      target: self.target.expect("build options.target failed"),
      resolve: self.resolve.expect("build options.resolve failed"),
      builtins: self.builtins.expect("build options.builtins failed"),
      plugins: self.plugins.expect("build options.plugins failed"),
      module: self.module.expect("build options.module failed"),
      devtool: self.devtool.expect("build options.devtool failed"),
      external: self.external.expect("build options.external failed"),
      external_type: self
        .external_type
        .expect("build options.external_type failed"),
      stats: self.stats.expect("build options.stats failed"),
      snapshot: self.snapshot.expect("build options.snapshot failed"),
      cache: self.cache.expect("build options.cache failed"),
      experiments: self.experiments.expect("build options.experiments failed"),
      node: self.node.expect("build options.node failed"),
      __emit_error: false,
      module_ids: self.module_ids.expect("build options.module_ids failed"),
      optimizations: self
        .optimizations
        .expect("build options.optimizations failed"),
    }
  }

  pub fn then<F: FnOnce(Self) -> anyhow::Result<Self>>(self, f: F) -> anyhow::Result<Self> {
    f(self)
  }
}
